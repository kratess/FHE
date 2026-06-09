/**
 * openfhe_wrapper.h
 *
 * A thin C-linkage shim that exposes the subset of the OpenFHE BGV API
 * needed by the Rust bindings. All heap-allocated objects are returned
 * as opaque void* handles; ownership is always transferred to the caller
 * and must be released with the corresponding *_free() function.
 *
 * Design rules:
 *   - No C++ types in the public signature (no std::, no lbcrypto::).
 *   - All error paths return NULL (for pointer returns) or set *ok = 0.
 *   - Thread-safety: each CryptoContext is independently usable; do not
 *     share a single context across threads without external locking.
 */
#pragma once
#ifdef __cplusplus
extern "C" {
#endif
#include <stdint.h>
#include <stddef.h>

/* ── Opaque handle types ─────────────────────────────────────────────────── */
typedef void* OFHEContext;    /* CryptoContext<DCRTPoly>         */
typedef void* OFHEPublicKey;  /* PublicKey<DCRTPoly>             */
typedef void* OFHEPrivateKey; /* PrivateKey<DCRTPoly>            */
typedef void* OFHEPlaintext;  /* Plaintext                       */
typedef void* OFHECiphertext; /* Ciphertext<DCRTPoly>            */

/* ── Raw byte buffers ───────────────────────────────────────────────────── */
void ofhe_bgv_buffer_free(uint8_t* buf);

/* ── Context lifecycle ───────────────────────────────────────────────────── */

/**
 * Create a BGV-RNS crypto context.
 *
 * @param plain_mod        Plaintext modulus t (must be a prime, e.g. 65537).
 * @param mult_depth       Maximum multiplicative depth.
 * @param batch_size       Number of SIMD slots (must be power-of-two ≤ n/2).
 * @param security_level   0 → HEStd_NotSet (fast), 128 → HEStd_128_classic.
 * @return  Opaque context handle, or NULL on failure.
 */
OFHEContext ofhe_bgv_context_new(uint64_t plain_mod, int mult_depth, int batch_size, int security_level);

void ofhe_bgv_context_free(OFHEContext ctx);

uint64_t ofhe_bgv_context_plain_mod(OFHEContext ctx);

size_t ofhe_bgv_ciphertext_serialized_size(OFHECiphertext ct);

int ofhe_bgv_ciphertext_serialize(OFHECiphertext ct, uint8_t* out, size_t out_len);

/* ── Key generation ──────────────────────────────────────────────────────── */

/**
 * Generate a fresh key pair.  Also generates the relinearisation key
 * (needed for EvalMult) and stores it inside the context.
 *
 * Caller owns both returned handles and must free them.
 */
void ofhe_bgv_keygen(OFHEContext ctx, OFHEPublicKey* out_pk, /* [out] */
                     OFHEPrivateKey* out_sk                  /* [out] */
);

void ofhe_bgv_pubkey_free(OFHEPublicKey pk);
void ofhe_bgv_seckey_free(OFHEPrivateKey sk);

/* ── Plaintext construction ──────────────────────────────────────────────── */

/**
 * Build a packed plaintext from a raw int64 array (SIMD slots).
 * Values are taken modulo plain_mod inside OpenFHE.
 */
OFHEPlaintext ofhe_bgv_make_packed_plaintext(OFHEContext ctx, const int64_t* values, size_t count);

void ofhe_bgv_plaintext_free(OFHEPlaintext pt);

/* ── Encrypt / Decrypt ───────────────────────────────────────────────────── */

OFHECiphertext ofhe_bgv_encrypt(OFHEContext ctx, OFHEPublicKey pk, OFHEPlaintext pt);

/**
 * Decrypt ct, write the first `out_len` slots into `out_values`.
 * Returns the number of slots actually written (≤ out_len), or -1 on error.
 */
int ofhe_bgv_decrypt(OFHEContext ctx, OFHEPrivateKey sk, OFHECiphertext ct, int64_t* out_values, size_t out_len);

void ofhe_bgv_ciphertext_free(OFHECiphertext ct);

/* ── Homomorphic arithmetic ──────────────────────────────────────────────── */

OFHECiphertext ofhe_bgv_eval_add(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b);
OFHECiphertext ofhe_bgv_eval_sub(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b);
OFHECiphertext ofhe_bgv_eval_mul(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b);

/** Multiply ciphertext by a *plaintext* scalar (no relin key needed). */
OFHECiphertext ofhe_bgv_eval_mul_plain(OFHEContext ctx, OFHECiphertext ct, OFHEPlaintext pt);

OFHECiphertext ofhe_bgv_eval_add_plain(OFHEContext ctx, OFHECiphertext ct, OFHEPlaintext pt);

OFHECiphertext ofhe_bgv_eval_sub_plain(OFHEContext ctx, OFHECiphertext ct, OFHEPlaintext pt);

/* ── Rotations ───────────────────────────────────────────────────────────── */

void ofhe_bgv_eval_rotate_keygen(OFHEContext ctx, OFHEPrivateKey sk, const int32_t* indices, size_t count);

OFHECiphertext ofhe_bgv_eval_rotate(OFHEContext ctx, OFHECiphertext ct, int32_t index);

/* ── EvalSum ─────────────────────────────────────────────────────────────── */

/**
 * Generate EvalSum keys (automorphism keys) and store them in the context.
 * Must be called before `ofhe_bgv_eval_sum`.
 */
void ofhe_bgv_eval_sum_keygen(OFHEContext ctx, OFHEPrivateKey sk);

/**
 * Sum all packed slots in `ct` (up to `batch_size`) and replicate the sum across slots.
 * Requires keys generated via `ofhe_bgv_eval_sum_keygen`.
 */
OFHECiphertext ofhe_bgv_eval_sum(OFHEContext ctx, OFHECiphertext ct, int32_t batch_size);

/* ── Serialization ───────────────────────────────────────────────────────── */

/**
 * Serialize a ciphertext to a newly allocated buffer.
 *
 * On success, returns 1 and sets (*out_buf, *out_len). Caller must free with
 * `ofhe_bgv_buffer_free`.
 */
int ofhe_bgv_serialize_ciphertext(OFHEContext ctx, OFHECiphertext ct, uint8_t** out_buf, size_t* out_len);

/**
 * Deserialize a ciphertext previously produced by `ofhe_bgv_serialize_ciphertext`.
 * Returns NULL on failure.
 */
OFHECiphertext ofhe_bgv_deserialize_ciphertext(OFHEContext ctx, const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_public_key(OFHEContext ctx, OFHEPublicKey pk, uint8_t** out_buf, size_t* out_len);

OFHEPublicKey ofhe_bgv_deserialize_public_key(OFHEContext ctx, const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_secret_key(OFHEContext ctx, OFHEPrivateKey sk, uint8_t** out_buf, size_t* out_len);

OFHEPrivateKey ofhe_bgv_deserialize_secret_key(OFHEContext ctx, const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_context(OFHEContext ctx, uint8_t** out_buf, size_t* out_len);

OFHEContext ofhe_bgv_deserialize_context(const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_eval_mult_key(OFHEContext ctx, uint8_t** out_buf, size_t* out_len);

int ofhe_bgv_deserialize_eval_mult_key(OFHEContext ctx, const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_eval_sum_key(OFHEContext ctx, uint8_t** out_buf, size_t* out_len);

int ofhe_bgv_deserialize_eval_sum_key(OFHEContext ctx, const uint8_t* buf, size_t len);

int ofhe_bgv_serialize_eval_rotate_key(OFHEContext ctx, uint8_t** out_buf, size_t* out_len);

int ofhe_bgv_deserialize_eval_rotate_key(OFHEContext ctx, const uint8_t* buf, size_t len);

#ifdef __cplusplus
}
#endif
