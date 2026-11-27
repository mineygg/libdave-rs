use std::os::raw::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use libdave_rs_sys as sys;

use crate::error::{DecryptError, EncryptError, Error, Result};
use crate::{Codec, MediaType};

/// Per-user media key ratchet (`DAVEKeyRatchetHandle`).
pub struct KeyRatchet {
    raw: sys::DAVEKeyRatchetHandle,
}

unsafe impl Send for KeyRatchet {}

impl KeyRatchet {
    pub(crate) fn from_raw(raw: sys::DAVEKeyRatchetHandle) -> Option<Self> {
        (!raw.is_null()).then_some(Self { raw })
    }
}

impl Drop for KeyRatchet {
    fn drop(&mut self) {
        unsafe { sys::daveKeyRatchetDestroy(self.raw) };
    }
}

/// Counters reported by an [`Encryptor`].
pub type EncryptorStats = sys::DAVEEncryptorStats;
/// Counters reported by a [`Decryptor`].
pub type DecryptorStats = sys::DAVEDecryptorStats;

/// Callback invoked when an [`Encryptor`]'s protocol version changes.
pub type ProtocolVersionHandler = Box<dyn FnMut() + Send>;

unsafe extern "C" fn protocol_version_trampoline(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    // SAFETY: `user_data` is the `Box<ProtocolVersionHandler>` owned by the `Encryptor`.
    let cb = unsafe { &mut *user_data.cast::<ProtocolVersionHandler>() };
    let _ = catch_unwind(AssertUnwindSafe(cb));
}

/// Media frame encryptor (`DAVEEncryptorHandle`).
pub struct Encryptor {
    raw: sys::DAVEEncryptorHandle,
    callback: *mut ProtocolVersionHandler,
}

unsafe impl Send for Encryptor {}

impl Encryptor {
    pub fn new() -> Result<Self> {
        let raw = unsafe { sys::daveEncryptorCreate() };
        if raw.is_null() {
            return Err(Error::CreateFailed("encryptor"));
        }
        Ok(Self {
            raw,
            callback: ptr::null_mut(),
        })
    }

    /// Installs the sender's key ratchet. libdave keeps its own reference, so the ratchet
    /// handle may be dropped afterwards.
    pub fn set_key_ratchet(&mut self, ratchet: &KeyRatchet) {
        unsafe { sys::daveEncryptorSetKeyRatchet(self.raw, ratchet.raw) };
    }

    pub fn set_passthrough_mode(&mut self, enabled: bool) {
        unsafe { sys::daveEncryptorSetPassthroughMode(self.raw, enabled) };
    }

    pub fn is_passthrough_mode(&self) -> bool {
        unsafe { sys::daveEncryptorIsPassthroughMode(self.raw) }
    }

    pub fn has_key_ratchet(&self) -> bool {
        unsafe { sys::daveEncryptorHasKeyRatchet(self.raw) }
    }

    pub fn protocol_version(&self) -> u16 {
        unsafe { sys::daveEncryptorGetProtocolVersion(self.raw) }
    }

    /// Installs (or removes, with `None`) a callback run whenever
    /// [`Encryptor::protocol_version`] changes.
    ///
    /// libdave calls it synchronously, from the method that caused the change.
    pub fn set_protocol_version_changed_callback(
        &mut self,
        handler: Option<ProtocolVersionHandler>,
    ) {
        let new = handler.map_or(ptr::null_mut(), |f| Box::into_raw(Box::new(f)));
        // libdave cannot unregister a callback, so removal passes null `user_data`,
        // which the trampoline ignores.
        unsafe {
            sys::daveEncryptorSetProtocolVersionChangedCallback(
                self.raw,
                protocol_version_trampoline,
                new.cast(),
            )
        };
        let old = std::mem::replace(&mut self.callback, new);
        if !old.is_null() {
            // SAFETY: libdave no longer holds `old`.
            drop(unsafe { Box::from_raw(old) });
        }
    }

    /// Tells the encryptor which codec an SSRC carries (needed for codec-aware framing).
    pub fn assign_ssrc_to_codec(&mut self, ssrc: u32, codec: Codec) {
        unsafe { sys::daveEncryptorAssignSsrcToCodec(self.raw, ssrc, codec as u32) };
    }

    pub fn max_ciphertext_size(&self, media: MediaType, frame_len: usize) -> usize {
        unsafe { sys::daveEncryptorGetMaxCiphertextByteSize(self.raw, media as u32, frame_len) }
    }

    /// Encrypts one frame into `out`, returning the number of bytes written.
    pub fn encrypt_into(
        &mut self,
        media: MediaType,
        ssrc: u32,
        frame: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<usize> {
        let cap = self.max_ciphertext_size(media, frame.len());
        out.clear();
        out.resize(cap, 0);
        let mut written = 0usize;
        let code = unsafe {
            sys::daveEncryptorEncrypt(
                self.raw,
                media as u32,
                ssrc,
                frame.as_ptr(),
                frame.len(),
                out.as_mut_ptr(),
                cap,
                &mut written,
            )
        };
        if code != sys::DAVE_ENCRYPTOR_RESULT_CODE_SUCCESS {
            out.clear();
            return Err(Error::Encrypt(EncryptError::from_code(code)));
        }
        out.truncate(written);
        Ok(written)
    }

    /// Convenience wrapper around [`Encryptor::encrypt_into`] that allocates.
    pub fn encrypt(&mut self, media: MediaType, ssrc: u32, frame: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.encrypt_into(media, ssrc, frame, &mut out)?;
        Ok(out)
    }

    pub fn stats(&self, media: MediaType) -> EncryptorStats {
        let mut s = EncryptorStats::default();
        unsafe { sys::daveEncryptorGetStats(self.raw, media as u32, &mut s) };
        s
    }
}

impl Drop for Encryptor {
    fn drop(&mut self) {
        unsafe { sys::daveEncryptorDestroy(self.raw) };
        if !self.callback.is_null() {
            drop(unsafe { Box::from_raw(self.callback) });
        }
    }
}

/// Media frame decryptor (`DAVEDecryptorHandle`), one per remote sender.
pub struct Decryptor {
    raw: sys::DAVEDecryptorHandle,
}

unsafe impl Send for Decryptor {}

impl Decryptor {
    pub fn new() -> Result<Self> {
        let raw = unsafe { sys::daveDecryptorCreate() };
        if raw.is_null() {
            return Err(Error::CreateFailed("decryptor"));
        }
        Ok(Self { raw })
    }

    /// Schedules a switch to `ratchet` (libdave keeps the old one for a grace period).
    pub fn transition_to_key_ratchet(&mut self, ratchet: &KeyRatchet) {
        unsafe { sys::daveDecryptorTransitionToKeyRatchet(self.raw, ratchet.raw) };
    }

    pub fn transition_to_passthrough_mode(&mut self, enabled: bool) {
        unsafe { sys::daveDecryptorTransitionToPassthroughMode(self.raw, enabled) };
    }

    pub fn max_plaintext_size(&self, media: MediaType, frame_len: usize) -> usize {
        unsafe { sys::daveDecryptorGetMaxPlaintextByteSize(self.raw, media as u32, frame_len) }
    }

    /// Decrypts one frame into `out`, returning the number of bytes written.
    pub fn decrypt_into(
        &mut self,
        media: MediaType,
        frame: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<usize> {
        let cap = self.max_plaintext_size(media, frame.len());
        out.clear();
        out.resize(cap, 0);
        let mut written = 0usize;
        let code = unsafe {
            sys::daveDecryptorDecrypt(
                self.raw,
                media as u32,
                frame.as_ptr(),
                frame.len(),
                out.as_mut_ptr(),
                cap,
                &mut written,
            )
        };
        if code != sys::DAVE_DECRYPTOR_RESULT_CODE_SUCCESS {
            out.clear();
            return Err(Error::Decrypt(DecryptError::from_code(code)));
        }
        out.truncate(written);
        Ok(written)
    }

    /// Convenience wrapper around [`Decryptor::decrypt_into`] that allocates.
    pub fn decrypt(&mut self, media: MediaType, frame: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.decrypt_into(media, frame, &mut out)?;
        Ok(out)
    }

    pub fn stats(&self, media: MediaType) -> DecryptorStats {
        let mut s = DecryptorStats::default();
        unsafe { sys::daveDecryptorGetStats(self.raw, media as u32, &mut s) };
        s
    }
}

impl Drop for Decryptor {
    fn drop(&mut self) {
        unsafe { sys::daveDecryptorDestroy(self.raw) };
    }
}
