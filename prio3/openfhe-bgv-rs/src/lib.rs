//! Raw FFI bindings to the OpenFHE BGV C++ shim.
//!
//! This crate is intentionally a thin porting layer only. It exposes the
//! C-linkage API from `wrapper/openfhe_wrapper.h` directly.
//!
//! For downstream crates that still expect the prior Rust API shape, this file
//! also exports a small compatibility layer built on top of the raw handles.

#![allow(non_camel_case_types, non_snake_case, dead_code, clippy::all)]

use std::ffi::c_int;
use std::fmt;
use std::fs;
use std::io;
use std::marker::PhantomData;
use std::os::raw::{c_longlong, c_void};
use std::path::Path;
use std::sync::Arc;

pub type OFHEContext = *mut c_void;
pub type OFHEPublicKey = *mut c_void;
pub type OFHEPrivateKey = *mut c_void;
pub type OFHEPlaintext = *mut c_void;
pub type OFHECiphertext = *mut c_void;

unsafe extern "C" {
    pub fn ofhe_bgv_buffer_free(buf: *mut u8);

    pub fn ofhe_bgv_context_new(
        plain_mod: u64,
        mult_depth: c_int,
        batch_size: c_int,
        security_level: c_int,
    ) -> OFHEContext;

    pub fn ofhe_bgv_context_free(ctx: OFHEContext);

    pub fn ofhe_bgv_context_plain_mod(ctx: OFHEContext) -> u64;

    pub fn ofhe_bgv_keygen(
        ctx: OFHEContext,
        out_pk: *mut OFHEPublicKey,
        out_sk: *mut OFHEPrivateKey,
    );

    pub fn ofhe_bgv_pubkey_free(pk: OFHEPublicKey);
    pub fn ofhe_bgv_seckey_free(sk: OFHEPrivateKey);

    pub fn ofhe_bgv_make_packed_plaintext(
        ctx: OFHEContext,
        values: *const c_longlong,
        count: usize,
    ) -> OFHEPlaintext;

    pub fn ofhe_bgv_plaintext_free(pt: OFHEPlaintext);

    pub fn ofhe_bgv_encrypt(
        ctx: OFHEContext,
        pk: OFHEPublicKey,
        pt: OFHEPlaintext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_decrypt(
        ctx: OFHEContext,
        sk: OFHEPrivateKey,
        ct: OFHECiphertext,
        out_values: *mut c_longlong,
        out_len: usize,
    ) -> c_int;

    pub fn ofhe_bgv_ciphertext_free(ct: OFHECiphertext);

    pub fn ofhe_bgv_ciphertext_serialized_size(ct: OFHECiphertext) -> usize;

    pub fn ofhe_bgv_ciphertext_serialize(ct: OFHECiphertext, out: *mut u8, out_len: usize)
        -> i32;

    pub fn ofhe_bgv_eval_add(
        ctx: OFHEContext,
        a: OFHECiphertext,
        b: OFHECiphertext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_sub(
        ctx: OFHEContext,
        a: OFHECiphertext,
        b: OFHECiphertext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_mul(
        ctx: OFHEContext,
        a: OFHECiphertext,
        b: OFHECiphertext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_mul_plain(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        pt: OFHEPlaintext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_add_plain(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        pt: OFHEPlaintext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_sub_plain(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        pt: OFHEPlaintext,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_rotate_keygen(
        ctx: OFHEContext,
        sk: OFHEPrivateKey,
        indices: *const i32,
        count: usize,
    );

    pub fn ofhe_bgv_eval_rotate(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        index: i32,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_eval_sum_keygen(ctx: OFHEContext, sk: OFHEPrivateKey);

    pub fn ofhe_bgv_eval_sum(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        batch_size: i32,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_serialize_ciphertext(
        ctx: OFHEContext,
        ct: OFHECiphertext,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_ciphertext(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> OFHECiphertext;

    pub fn ofhe_bgv_serialize_public_key(
        ctx: OFHEContext,
        pk: OFHEPublicKey,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_public_key(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> OFHEPublicKey;

    pub fn ofhe_bgv_serialize_secret_key(
        ctx: OFHEContext,
        sk: OFHEPrivateKey,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_secret_key(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> OFHEPrivateKey;

    pub fn ofhe_bgv_serialize_context(
        ctx: OFHEContext,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_context(buf: *const u8, len: usize) -> OFHEContext;

    pub fn ofhe_bgv_serialize_eval_mult_key(
        ctx: OFHEContext,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_eval_mult_key(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> c_int;

    pub fn ofhe_bgv_serialize_eval_sum_key(
        ctx: OFHEContext,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_eval_sum_key(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> c_int;

    pub fn ofhe_bgv_serialize_eval_rotate_key(
        ctx: OFHEContext,
        out_buf: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;

    pub fn ofhe_bgv_deserialize_eval_rotate_key(
        ctx: OFHEContext,
        buf: *const u8,
        len: usize,
    ) -> c_int;
}

pub trait BgvElement {
    fn to_i64(&self) -> i64;
    fn from_u64(val: u64, plain_mod: u64) -> Self;
}

macro_rules! impl_bgv_element_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl BgvElement for $t {
                fn to_i64(&self) -> i64 {
                    *self as i64
                }

                fn from_u64(val: u64, _plain_mod: u64) -> Self {
                    val as Self
                }
            }
        )*
    };
}

impl_bgv_element_int!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgvError {
    ContextCreation,
    Serialization,
    Io,
    PlaintextCreation,
    Encryption,
    Decryption,
    EvalAdd,
    EvalSub,
    EvalMul,
    EvalRotate,
    EvalSum,
}

impl fmt::Display for BgvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BgvError::ContextCreation => write!(f, "failed to create BGV context"),
            BgvError::Serialization => write!(f, "BGV serialization failed"),
            BgvError::Io => write!(f, "BGV artifact I/O failed"),
            BgvError::PlaintextCreation => write!(f, "failed to create plaintext"),
            BgvError::Encryption => write!(f, "encryption failed"),
            BgvError::Decryption => write!(f, "decryption failed"),
            BgvError::EvalAdd => write!(f, "homomorphic addition failed"),
            BgvError::EvalSub => write!(f, "homomorphic subtraction failed"),
            BgvError::EvalMul => write!(f, "homomorphic multiplication failed"),
            BgvError::EvalRotate => write!(f, "homomorphic rotation failed"),
            BgvError::EvalSum => write!(f, "homomorphic sum failed"),
        }
    }
}

impl std::error::Error for BgvError {}

impl From<io::Error> for BgvError {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}

pub type Result<T> = std::result::Result<T, BgvError>;

#[derive(Debug, Clone)]
pub struct BgvParams {
    pub plain_mod: u64,
    pub mult_depth: i32,
    pub batch_size: i32,
    pub security_level: i32,
}

#[derive(Debug, Clone)]
pub struct BgvContext(Arc<ContextInner>);

#[derive(Debug)]
struct ContextInner {
    ptr: OFHEContext,
    plain_mod: u64,
}

unsafe impl Send for ContextInner {}
unsafe impl Sync for ContextInner {}

impl Drop for ContextInner {
    fn drop(&mut self) {
        unsafe { ofhe_bgv_context_free(self.ptr) };
    }
}

impl BgvContext {
    pub fn new(params: BgvParams) -> Result<Self> {
        let ptr = unsafe {
            ofhe_bgv_context_new(
                params.plain_mod,
                params.mult_depth,
                params.batch_size,
                params.security_level,
            )
        };
        if ptr.is_null() {
            return Err(BgvError::ContextCreation);
        }
        Ok(Self(Arc::new(ContextInner {
            ptr,
            plain_mod: params.plain_mod,
        })))
    }

    pub fn plain_mod(&self) -> u64 {
        self.0.plain_mod
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_context(self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let ptr = unsafe { ofhe_bgv_deserialize_context(bytes.as_ptr(), bytes.len()) };
        if ptr.is_null() {
            return Err(BgvError::ContextCreation);
        }
        let plain_mod = unsafe { ofhe_bgv_context_plain_mod(ptr) };
        if plain_mod == 0 {
            unsafe { ofhe_bgv_context_free(ptr) };
            return Err(BgvError::ContextCreation);
        }
        Ok(Self(Arc::new(ContextInner { ptr, plain_mod })))
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize()?)?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = fs::read(path)?;
        Self::deserialize(&bytes)
    }

    pub fn keygen(&self) -> (PublicKey, SecretKey) {
        let mut raw_pk: OFHEPublicKey = std::ptr::null_mut();
        let mut raw_sk: OFHEPrivateKey = std::ptr::null_mut();
        unsafe {
            ofhe_bgv_keygen(self.0.ptr, &mut raw_pk, &mut raw_sk);
        }
        (
            PublicKey(Arc::new(PublicKeyInner { ptr: raw_pk })),
            SecretKey(Arc::new(SecretKeyInner { ptr: raw_sk })),
        )
    }

    pub fn make_plaintext<G: BgvElement>(&self, values: &[G]) -> Result<Plaintext> {
        let i64_values: Vec<i64> = values.iter().map(|v| v.to_i64()).collect();
        let ptr =
            unsafe { ofhe_bgv_make_packed_plaintext(self.0.ptr, i64_values.as_ptr(), i64_values.len()) };
        if ptr.is_null() {
            return Err(BgvError::PlaintextCreation);
        }
        Ok(Plaintext(Arc::new(PlaintextInner { ptr })))
    }

    pub fn encrypt<G>(&self, pk: &PublicKey, pt: &Plaintext) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_encrypt(self.0.ptr, pk.0.ptr, pt.0.ptr) };
        non_null(ptr, BgvError::Encryption)
    }

    pub fn decrypt<G: BgvElement>(
        &self,
        sk: &SecretKey,
        ct: &Ciphertext<G>,
        slots: usize,
    ) -> Result<Vec<G>> {
        let mut out = vec![0i64; slots];
        let written =
            unsafe { ofhe_bgv_decrypt(self.0.ptr, sk.0.ptr, ct.0.ptr, out.as_mut_ptr(), slots) };
        if written < 0 {
            return Err(BgvError::Decryption);
        }
        let plain_mod = self.plain_mod();
        Ok(out
            .into_iter()
            .take(written as usize)
            .map(|v| G::from_u64(v as u64, plain_mod))
            .collect())
    }

    pub fn eval_add<G>(&self, a: &Ciphertext<G>, b: &Ciphertext<G>) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_add(self.0.ptr, a.0.ptr, b.0.ptr) };
        non_null(ptr, BgvError::EvalAdd)
    }

    pub fn eval_sub<G>(&self, a: &Ciphertext<G>, b: &Ciphertext<G>) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_sub(self.0.ptr, a.0.ptr, b.0.ptr) };
        non_null(ptr, BgvError::EvalSub)
    }

    pub fn eval_mul<G>(&self, a: &Ciphertext<G>, b: &Ciphertext<G>) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_mul(self.0.ptr, a.0.ptr, b.0.ptr) };
        non_null(ptr, BgvError::EvalMul)
    }

    pub fn eval_mul_plain<G>(&self, ct: &Ciphertext<G>, pt: &Plaintext) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_mul_plain(self.0.ptr, ct.0.ptr, pt.0.ptr) };
        non_null(ptr, BgvError::EvalMul)
    }

    pub fn eval_add_plain<G>(&self, ct: &Ciphertext<G>, pt: &Plaintext) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_add_plain(self.0.ptr, ct.0.ptr, pt.0.ptr) };
        non_null(ptr, BgvError::EvalAdd)
    }

    pub fn eval_sub_plain<G>(&self, ct: &Ciphertext<G>, pt: &Plaintext) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_sub_plain(self.0.ptr, ct.0.ptr, pt.0.ptr) };
        non_null(ptr, BgvError::EvalSub)
    }

    pub fn eval_rotate_keygen(&self, sk: &SecretKey, indices: &[i32]) {
        unsafe { ofhe_bgv_eval_rotate_keygen(self.0.ptr, sk.0.ptr, indices.as_ptr(), indices.len()) };
    }

    pub fn eval_rotate<G>(&self, ct: &Ciphertext<G>, index: i32) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_rotate(self.0.ptr, ct.0.ptr, index) };
        non_null(ptr, BgvError::EvalRotate)
    }

    pub fn eval_sum_keygen(&self, sk: &SecretKey) {
        unsafe { ofhe_bgv_eval_sum_keygen(self.0.ptr, sk.0.ptr) };
    }

    pub fn eval_sum<G>(&self, ct: &Ciphertext<G>, batch_size: i32) -> Result<Ciphertext<G>> {
        let ptr = unsafe { ofhe_bgv_eval_sum(self.0.ptr, ct.0.ptr, batch_size) };
        non_null(ptr, BgvError::EvalSum)
    }

    pub fn serialize_eval_mult_key(&self) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_eval_mult_key(self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize_eval_mult_key(&self, bytes: &[u8]) -> Result<()> {
        deserialize_unit(unsafe {
            ofhe_bgv_deserialize_eval_mult_key(self.0.ptr, bytes.as_ptr(), bytes.len())
        })
    }

    pub fn serialize_eval_sum_key(&self) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_eval_sum_key(self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize_eval_sum_key(&self, bytes: &[u8]) -> Result<()> {
        deserialize_unit(unsafe {
            ofhe_bgv_deserialize_eval_sum_key(self.0.ptr, bytes.as_ptr(), bytes.len())
        })
    }

    pub fn serialize_eval_rotate_key(&self) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_eval_rotate_key(self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize_eval_rotate_key(&self, bytes: &[u8]) -> Result<()> {
        deserialize_unit(unsafe {
            ofhe_bgv_deserialize_eval_rotate_key(self.0.ptr, bytes.as_ptr(), bytes.len())
        })
    }

    pub fn save_eval_mult_key_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize_eval_mult_key()?)?;
        Ok(())
    }

    pub fn load_eval_mult_key_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.deserialize_eval_mult_key(&fs::read(path)?)
    }

    pub fn save_eval_sum_key_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize_eval_sum_key()?)?;
        Ok(())
    }

    pub fn load_eval_sum_key_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.deserialize_eval_sum_key(&fs::read(path)?)
    }

    pub fn save_eval_rotate_key_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize_eval_rotate_key()?)?;
        Ok(())
    }

    pub fn load_eval_rotate_key_from_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.deserialize_eval_rotate_key(&fs::read(path)?)
    }
}

#[derive(Debug, Clone)]
pub struct PublicKey(Arc<PublicKeyInner>);

#[derive(Debug)]
struct PublicKeyInner {
    ptr: OFHEPublicKey,
}

unsafe impl Send for PublicKeyInner {}
unsafe impl Sync for PublicKeyInner {}

impl Drop for PublicKeyInner {
    fn drop(&mut self) {
        unsafe { ofhe_bgv_pubkey_free(self.ptr) };
    }
}

impl PublicKey {
    pub fn serialize(&self, ctx: &BgvContext) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_public_key(ctx.0.ptr, self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize(ctx: &BgvContext, bytes: &[u8]) -> Result<Self> {
        let ptr =
            unsafe { ofhe_bgv_deserialize_public_key(ctx.0.ptr, bytes.as_ptr(), bytes.len()) };
        if ptr.is_null() {
            return Err(BgvError::Serialization);
        }
        Ok(Self(Arc::new(PublicKeyInner { ptr })))
    }

    pub fn save_to_file(&self, ctx: &BgvContext, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize(ctx)?)?;
        Ok(())
    }

    pub fn load_from_file(ctx: &BgvContext, path: impl AsRef<Path>) -> Result<Self> {
        Self::deserialize(ctx, &fs::read(path)?)
    }
}

#[derive(Debug, Clone)]
pub struct SecretKey(Arc<SecretKeyInner>);

#[derive(Debug)]
struct SecretKeyInner {
    ptr: OFHEPrivateKey,
}

unsafe impl Send for SecretKeyInner {}
unsafe impl Sync for SecretKeyInner {}

impl Drop for SecretKeyInner {
    fn drop(&mut self) {
        unsafe { ofhe_bgv_seckey_free(self.ptr) };
    }
}

impl SecretKey {
    pub fn serialize(&self, ctx: &BgvContext) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_secret_key(ctx.0.ptr, self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize(ctx: &BgvContext, bytes: &[u8]) -> Result<Self> {
        let ptr =
            unsafe { ofhe_bgv_deserialize_secret_key(ctx.0.ptr, bytes.as_ptr(), bytes.len()) };
        if ptr.is_null() {
            return Err(BgvError::Serialization);
        }
        Ok(Self(Arc::new(SecretKeyInner { ptr })))
    }

    pub fn save_to_file(&self, ctx: &BgvContext, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize(ctx)?)?;
        Ok(())
    }

    pub fn load_from_file(ctx: &BgvContext, path: impl AsRef<Path>) -> Result<Self> {
        Self::deserialize(ctx, &fs::read(path)?)
    }
}

#[derive(Debug, Clone)]
pub struct Plaintext(Arc<PlaintextInner>);

#[derive(Debug)]
struct PlaintextInner {
    ptr: OFHEPlaintext,
}

unsafe impl Send for PlaintextInner {}
unsafe impl Sync for PlaintextInner {}

impl Drop for PlaintextInner {
    fn drop(&mut self) {
        unsafe { ofhe_bgv_plaintext_free(self.ptr) };
    }
}

#[derive(Debug, Clone)]
pub struct Ciphertext<G>(Arc<CiphertextInner>, PhantomData<G>);

#[derive(Debug)]
struct CiphertextInner {
    ptr: OFHECiphertext,
}

unsafe impl Send for CiphertextInner {}
unsafe impl Sync for CiphertextInner {}

impl Drop for CiphertextInner {
    fn drop(&mut self) {
        unsafe { ofhe_bgv_ciphertext_free(self.ptr) };
    }
}

impl<G> Ciphertext<G> {
    pub fn serialize(&self, ctx: &BgvContext) -> Result<Vec<u8>> {
        serialize_with(|out_buf, out_len| unsafe {
            ofhe_bgv_serialize_ciphertext(ctx.0.ptr, self.0.ptr, out_buf, out_len)
        })
    }

    pub fn deserialize(ctx: &BgvContext, bytes: &[u8]) -> Result<Self> {
        let ptr =
            unsafe { ofhe_bgv_deserialize_ciphertext(ctx.0.ptr, bytes.as_ptr(), bytes.len()) };
        non_null(ptr, BgvError::Serialization)
    }

    pub fn save_to_file(&self, ctx: &BgvContext, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, self.serialize(ctx)?)?;
        Ok(())
    }

    pub fn load_from_file(ctx: &BgvContext, path: impl AsRef<Path>) -> Result<Self> {
        Self::deserialize(ctx, &fs::read(path)?)
    }
}

fn non_null<G>(ptr: OFHECiphertext, err: BgvError) -> Result<Ciphertext<G>> {
    if ptr.is_null() {
        Err(err)
    } else {
        Ok(Ciphertext(Arc::new(CiphertextInner { ptr }), PhantomData))
    }
}

fn deserialize_unit(result: c_int) -> Result<()> {
    if result == 1 {
        Ok(())
    } else {
        Err(BgvError::Serialization)
    }
}

fn serialize_with<F>(f: F) -> Result<Vec<u8>>
where
    F: FnOnce(*mut *mut u8, *mut usize) -> c_int,
{
    let mut out_buf: *mut u8 = std::ptr::null_mut();
    let mut out_len = 0usize;
    let status = f(&mut out_buf, &mut out_len);
    if status != 1 || out_buf.is_null() || out_len == 0 {
        return Err(BgvError::Serialization);
    }
    let bytes = unsafe { Vec::from_raw_parts(out_buf, out_len, out_len) };
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{BgvContext, BgvParams};
    use std::sync::{Mutex, OnceLock};

    fn openfhe_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn lock_guard() -> std::sync::MutexGuard<'static, ()> {
        match openfhe_test_lock().lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn test_params() -> BgvParams {
        BgvParams {
            plain_mod: 786433,
            mult_depth: 24,
            batch_size: 128,
            security_level: 128,
        }
    }

    #[test]
    fn context_and_key_roundtrip() {
        let _guard = lock_guard();
        let ctx = BgvContext::new(test_params()).unwrap();
        let (pk, sk) = ctx.keygen();
        ctx.eval_sum_keygen(&sk);
        ctx.eval_rotate_keygen(&sk, &[-1, -2, -3]);

        let ctx_bytes = ctx.serialize().unwrap();
        let ctx2 = BgvContext::deserialize(&ctx_bytes).unwrap();
        assert_eq!(ctx2.plain_mod(), ctx.plain_mod());

        let pk_bytes = pk.serialize(&ctx).unwrap();
        let _pk2 = super::PublicKey::deserialize(&ctx2, &pk_bytes).unwrap();

        let sk_bytes = sk.serialize(&ctx).unwrap();
        let sk2 = super::SecretKey::deserialize(&ctx2, &sk_bytes).unwrap();

        let pt = ctx.make_plaintext(&[1usize, 0, 1, 1]).unwrap();
        let ct: super::Ciphertext<usize> = ctx.encrypt(&pk, &pt).unwrap();
        let ct_bytes = ct.serialize(&ctx).unwrap();
        let ct2 = super::Ciphertext::<usize>::deserialize(&ctx2, &ct_bytes).unwrap();
        let decoded = ctx2.decrypt::<usize>(&sk2, &ct2, 4).unwrap();
        assert_eq!(decoded, vec![1, 0, 1, 1]);
    }

    #[test]
    fn eval_key_roundtrip() {
        let _guard = lock_guard();
        let ctx = BgvContext::new(test_params()).unwrap();
        let (pk, sk) = ctx.keygen();
        ctx.eval_sum_keygen(&sk);
        ctx.eval_rotate_keygen(&sk, &[-1, -2, -3]);

        let eval_mult = ctx.serialize_eval_mult_key().unwrap();
        let eval_sum = ctx.serialize_eval_sum_key().unwrap();
        let eval_rotate = ctx.serialize_eval_rotate_key().unwrap();

        let ctx2 = BgvContext::deserialize(&ctx.serialize().unwrap()).unwrap();
        let pk2 = super::PublicKey::deserialize(&ctx2, &pk.serialize(&ctx).unwrap()).unwrap();
        let sk2 = super::SecretKey::deserialize(&ctx2, &sk.serialize(&ctx).unwrap()).unwrap();
        ctx2.deserialize_eval_mult_key(&eval_mult).unwrap();
        ctx2.deserialize_eval_sum_key(&eval_sum).unwrap();
        ctx2.deserialize_eval_rotate_key(&eval_rotate).unwrap();

        let pt = ctx2.make_plaintext(&[1usize, 0, 1, 1]).unwrap();
        let ct: super::Ciphertext<usize> = ctx2.encrypt(&pk2, &pt).unwrap();
        let summed = ctx2.eval_sum(&ct, 4).unwrap();
        let decoded = ctx2.decrypt::<usize>(&sk2, &summed, 4).unwrap();
        assert_eq!(decoded[0], 3);
    }
}
