/**
 * openfhe_wrapper.cpp
 *
 * Implements the C-linkage shim declared in openfhe_wrapper.h.
 * Each handle is a heap-allocated object; ownership is documented
 * in the header.  Exceptions from OpenFHE are caught and converted
 * to NULL / -1 return values so the Rust side never sees a C++ exception.
 */
#include "openfhe_wrapper.h"

#include <cstdint>
#include <cstring>
#include <memory>
#include <stdexcept>
#include <string>
#include <sstream>
#include <vector>

// OpenFHE umbrella header
#include "openfhe.h"
// Serialization support for CryptoContext / keys / ciphertext.
#include "openfhe/pke/cryptocontext-ser.h"
using namespace lbcrypto;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------
namespace {

// We store the CryptoContext together with the key-pair so that
// EvalMultKeyGen results (relin keys) are accessible from the context.
struct ContextBundle {
    CryptoContext<DCRTPoly> cc;
};

// Thin wrappers so we can new/delete cleanly via void*.
struct PubKeyBundle  { PublicKey<DCRTPoly>  key; };
struct SecKeyBundle  { PrivateKey<DCRTPoly> key; };
struct PlainBundle   { Plaintext            pt;  };
struct CiphBundle    { Ciphertext<DCRTPoly> ct;  };

}  // namespace

// ---------------------------------------------------------------------------
// Raw buffers
// ---------------------------------------------------------------------------
extern "C" void ofhe_bgv_buffer_free(uint8_t* buf) { delete[] buf; }

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------
extern "C"
OFHEContext ofhe_bgv_context_new(
    uint64_t plain_mod,
    int      mult_depth,
    int      batch_size,
    int      security_level)
{
    try {
        CCParams<CryptoContextBGVRNS> params;
        params.SetPlaintextModulus(plain_mod);
        params.SetMultiplicativeDepth(mult_depth);
        params.SetBatchSize(batch_size);

        switch (security_level) {
            case 128: params.SetSecurityLevel(HEStd_128_classic); break;
            case 192: params.SetSecurityLevel(HEStd_192_classic); break;
            case 256: params.SetSecurityLevel(HEStd_256_classic); break;
            default:  params.SetSecurityLevel(HEStd_NotSet);      break;
        }

        auto* bundle = new ContextBundle();
        bundle->cc   = GenCryptoContext(params);
        bundle->cc->Enable(PKE);
        bundle->cc->Enable(KEYSWITCH);
        bundle->cc->Enable(LEVELEDSHE);
        bundle->cc->Enable(ADVANCEDSHE);
        return static_cast<void*>(bundle);
    } catch (...) {
        return nullptr;
    }
}

extern "C"
void ofhe_bgv_context_free(OFHEContext ctx) {
    delete static_cast<ContextBundle*>(ctx);
}

extern "C"
size_t ofhe_bgv_ciphertext_serialized_size(OFHECiphertext ct) {
    try {
        auto* ciphertext = static_cast<lbcrypto::Ciphertext<lbcrypto::DCRTPoly>*>(ct);

        std::stringstream ss;
        Serial::Serialize(*ciphertext, ss, SerType::BINARY);

        return ss.str().size();
    } catch (...) {
        return 0;
    }
}

extern "C"
int ofhe_bgv_ciphertext_serialize(
    OFHECiphertext ct,
    uint8_t* out,
    size_t out_len
) {
    try {
        auto* ciphertext = static_cast<lbcrypto::Ciphertext<lbcrypto::DCRTPoly>*>(ct);

        std::stringstream ss;
        Serial::Serialize(*ciphertext, ss, SerType::BINARY);

        std::string data = ss.str();

        if (out_len < data.size()) {
            return -1; // buffer too small
        }

        std::memcpy(out, data.data(), data.size());
        return static_cast<int>(data.size());
    } catch (...) {
        return -2;
    }
}

// ---------------------------------------------------------------------------
// Key generation
// ---------------------------------------------------------------------------
extern "C"
void ofhe_bgv_keygen(
    OFHEContext     ctx,
    OFHEPublicKey*  out_pk,
    OFHEPrivateKey* out_sk)
{
    *out_pk = nullptr;
    *out_sk = nullptr;
    if (!ctx) return;
    try {
        auto& cc = static_cast<ContextBundle*>(ctx)->cc;
        auto kp  = cc->KeyGen();
        cc->EvalMultKeyGen(kp.secretKey);  // relin key stored in cc

        auto* pk_b = new PubKeyBundle();
        pk_b->key  = kp.publicKey;
        auto* sk_b = new SecKeyBundle();
        sk_b->key  = kp.secretKey;

        *out_pk = static_cast<void*>(pk_b);
        *out_sk = static_cast<void*>(sk_b);
    } catch (...) {}
}

extern "C" void ofhe_bgv_pubkey_free(OFHEPublicKey pk)  { delete static_cast<PubKeyBundle*>(pk);  }
extern "C" void ofhe_bgv_seckey_free(OFHEPrivateKey sk)  { delete static_cast<SecKeyBundle*>(sk);  }

// ---------------------------------------------------------------------------
// Plaintext
// ---------------------------------------------------------------------------
extern "C"
OFHEPlaintext ofhe_bgv_make_packed_plaintext(
    OFHEContext   ctx,
    const int64_t* values,
    size_t         count)
{
    if (!ctx || !values || count == 0) return nullptr;
    try {
        auto& cc = static_cast<ContextBundle*>(ctx)->cc;
        std::vector<int64_t> vec(values, values + count);
        auto* b  = new PlainBundle();
        b->pt    = cc->MakePackedPlaintext(vec);
        return static_cast<void*>(b);
    } catch (...) {
        return nullptr;
    }
}

extern "C" void ofhe_bgv_plaintext_free(OFHEPlaintext pt) { delete static_cast<PlainBundle*>(pt); }

// ---------------------------------------------------------------------------
// Encrypt / Decrypt
// ---------------------------------------------------------------------------
extern "C"
OFHECiphertext ofhe_bgv_encrypt(
    OFHEContext   ctx,
    OFHEPublicKey pk,
    OFHEPlaintext pt)
{
    if (!ctx || !pk || !pt) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& key  = static_cast<PubKeyBundle*>(pk)->key;
        auto& pobj = static_cast<PlainBundle*>(pt)->pt;
        auto* b    = new CiphBundle();
        b->ct      = cc->Encrypt(key, pobj);
        return static_cast<void*>(b);
    } catch (...) {
        return nullptr;
    }
}

extern "C"
int ofhe_bgv_decrypt(
    OFHEContext    ctx,
    OFHEPrivateKey sk,
    OFHECiphertext ct,
    int64_t*       out_values,
    size_t         out_len)
{
    if (!ctx || !sk || !ct || !out_values || out_len == 0) return -1;
    try {
        auto& cc    = static_cast<ContextBundle*>(ctx)->cc;
        auto& skey  = static_cast<SecKeyBundle*>(sk)->key;
        auto& ciph  = static_cast<CiphBundle*>(ct)->ct;
        Plaintext pt;
        cc->Decrypt(skey, ciph, &pt);
        pt->SetLength(out_len);
        const auto& packed = pt->GetPackedValue();
        size_t n = std::min(packed.size(), out_len);
        for (size_t i = 0; i < n; ++i)
            out_values[i] = packed[i];
        return static_cast<int>(n);
    } catch (...) {
        return -1;
    }
}

extern "C" void ofhe_bgv_ciphertext_free(OFHECiphertext ct) { delete static_cast<CiphBundle*>(ct); }

// ---------------------------------------------------------------------------
// Homomorphic arithmetic helpers
// ---------------------------------------------------------------------------
namespace {
template<typename Op>
OFHECiphertext binary_op(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b, Op op) {
    if (!ctx || !a || !b) return nullptr;
    try {
        auto& cc = static_cast<ContextBundle*>(ctx)->cc;
        auto* res = new CiphBundle();
        res->ct = op(cc,
                     static_cast<CiphBundle*>(a)->ct,
                     static_cast<CiphBundle*>(b)->ct);
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}
}

extern "C"
OFHECiphertext ofhe_bgv_eval_add(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b) {
    return binary_op(ctx, a, b, [](auto& cc, auto& x, auto& y){ return cc->EvalAdd(x, y); });
}

extern "C"
OFHECiphertext ofhe_bgv_eval_sub(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b) {
    return binary_op(ctx, a, b, [](auto& cc, auto& x, auto& y){ return cc->EvalSub(x, y); });
}

extern "C"
OFHECiphertext ofhe_bgv_eval_mul(OFHEContext ctx, OFHECiphertext a, OFHECiphertext b) {
    return binary_op(ctx, a, b, [](auto& cc, auto& x, auto& y){ return cc->EvalMult(x, y); });
}

extern "C"
OFHECiphertext ofhe_bgv_eval_mul_plain(
    OFHEContext    ctx,
    OFHECiphertext ct,
    OFHEPlaintext  pt)
{
    if (!ctx || !ct || !pt) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& ciph = static_cast<CiphBundle*>(ct)->ct;
        auto& pobj = static_cast<PlainBundle*>(pt)->pt;
        auto* res  = new CiphBundle();
        res->ct    = cc->EvalMult(ciph, pobj);
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}

extern "C"
OFHECiphertext ofhe_bgv_eval_add_plain(
    OFHEContext    ctx,
    OFHECiphertext ct,
    OFHEPlaintext  pt)
{
    if (!ctx || !ct || !pt) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& ciph = static_cast<CiphBundle*>(ct)->ct;
        auto& pobj = static_cast<PlainBundle*>(pt)->pt;
        auto* res  = new CiphBundle();
        res->ct    = cc->EvalAdd(ciph, pobj);
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}

extern "C"
OFHECiphertext ofhe_bgv_eval_sub_plain(
    OFHEContext    ctx,
    OFHECiphertext ct,
    OFHEPlaintext  pt)
{
    if (!ctx || !ct || !pt) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& ciph = static_cast<CiphBundle*>(ct)->ct;
        auto& pobj = static_cast<PlainBundle*>(pt)->pt;
        auto* res  = new CiphBundle();
        res->ct    = cc->EvalSub(ciph, pobj);
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}

// ---------------------------------------------------------------------------
// Rotations
// ---------------------------------------------------------------------------

extern "C"
void ofhe_bgv_eval_rotate_keygen(
    OFHEContext    ctx,
    OFHEPrivateKey sk,
    const int32_t* indices,
    size_t         count)
{
    if (!ctx || !sk || !indices || count == 0) return;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& skey = static_cast<SecKeyBundle*>(sk)->key;
        std::vector<int32_t> index_vec(indices, indices + count);
        cc->EvalRotateKeyGen(skey, index_vec);
    } catch (...) {}
}

extern "C"
OFHECiphertext ofhe_bgv_eval_rotate(
    OFHEContext    ctx,
    OFHECiphertext ct,
    int32_t        index)
{
    if (!ctx || !ct) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& ciph = static_cast<CiphBundle*>(ct)->ct;
        auto* res  = new CiphBundle();
        res->ct    = cc->EvalRotate(ciph, index);
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}

// ---------------------------------------------------------------------------
// EvalSum
// ---------------------------------------------------------------------------

extern "C"
void ofhe_bgv_eval_sum_keygen(OFHEContext ctx, OFHEPrivateKey sk) {
    if (!ctx || !sk) return;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& skey = static_cast<SecKeyBundle*>(sk)->key;
        cc->EvalSumKeyGen(skey);
    } catch (...) {}
}

extern "C"
OFHECiphertext ofhe_bgv_eval_sum(
    OFHEContext    ctx,
    OFHECiphertext ct,
    int32_t        batch_size)
{
    if (!ctx || !ct || batch_size <= 0) return nullptr;
    try {
        auto& cc   = static_cast<ContextBundle*>(ctx)->cc;
        auto& ciph = static_cast<CiphBundle*>(ct)->ct;
        auto* res  = new CiphBundle();
        res->ct    = cc->EvalSum(ciph, static_cast<uint32_t>(batch_size));
        return static_cast<void*>(res);
    } catch (...) { return nullptr; }
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

namespace {
template <class T>
int serialize_obj(const T& obj, uint8_t** out_buf, size_t* out_len) {
    if (!out_buf || !out_len) return 0;
    *out_buf = nullptr;
    *out_len = 0;
    try {
        std::stringstream ss;
        Serial::Serialize(obj, ss, SerType::BINARY);
        const std::string s = ss.str();
        if (s.empty()) return 0;
        auto* buf = new uint8_t[s.size()];
        std::memcpy(buf, s.data(), s.size());
        *out_buf = buf;
        *out_len = s.size();
        return 1;
    } catch (...) {
        return 0;
    }
}

template <class T>
bool deserialize_obj(T* out, const uint8_t* buf, size_t len) {
    if (!out || !buf || len == 0) return false;
    try {
        std::string s(reinterpret_cast<const char*>(buf), len);
        std::stringstream ss(s);
        Serial::Deserialize(*out, ss, SerType::BINARY);
        return true;
    } catch (...) {
        return false;
    }
}
}  // namespace

extern "C"
int ofhe_bgv_serialize_ciphertext(
    OFHEContext     ctx,
    OFHECiphertext  ct,
    uint8_t**       out_buf,
    size_t*         out_len)
{
    (void)ctx;
    if (!ct) return 0;
    return serialize_obj(static_cast<CiphBundle*>(ct)->ct, out_buf, out_len);
}

extern "C"
OFHECiphertext ofhe_bgv_deserialize_ciphertext(
    OFHEContext     ctx,
    const uint8_t*  buf,
    size_t          len)
{
    if (!ctx || !buf || len == 0) return nullptr;
    try {
        auto* b = new CiphBundle();
        if (!deserialize_obj(&b->ct, buf, len)) {
            delete b;
            return nullptr;
        }
        return static_cast<void*>(b);
    } catch (...) {
        return nullptr;
    }
}

extern "C"
int ofhe_bgv_serialize_public_key(
    OFHEContext     ctx,
    OFHEPublicKey   pk,
    uint8_t**       out_buf,
    size_t*         out_len)
{
    (void)ctx;
    if (!pk) return 0;
    return serialize_obj(static_cast<PubKeyBundle*>(pk)->key, out_buf, out_len);
}

extern "C"
OFHEPublicKey ofhe_bgv_deserialize_public_key(
    OFHEContext     ctx,
    const uint8_t*  buf,
    size_t          len)
{
    if (!ctx || !buf || len == 0) return nullptr;
    try {
        auto* b = new PubKeyBundle();
        if (!deserialize_obj(&b->key, buf, len)) {
            delete b;
            return nullptr;
        }
        return static_cast<void*>(b);
    } catch (...) {
        return nullptr;
    }
}

extern "C"
int ofhe_bgv_serialize_secret_key(
    OFHEContext     ctx,
    OFHEPrivateKey  sk,
    uint8_t**       out_buf,
    size_t*         out_len)
{
    (void)ctx;
    if (!sk) return 0;
    return serialize_obj(static_cast<SecKeyBundle*>(sk)->key, out_buf, out_len);
}

extern "C"
OFHEPrivateKey ofhe_bgv_deserialize_secret_key(
    OFHEContext     ctx,
    const uint8_t*  buf,
    size_t          len)
{
    if (!ctx || !buf || len == 0) return nullptr;
    try {
        auto* b = new SecKeyBundle();
        if (!deserialize_obj(&b->key, buf, len)) {
            delete b;
            return nullptr;
        }
        return static_cast<void*>(b);
    } catch (...) {
        return nullptr;
    }
}
