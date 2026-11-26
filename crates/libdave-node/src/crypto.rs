use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::types::{Codec, DecryptStats, EncryptStats, MediaType};
use crate::{destroyed, to_napi};

/// Per-user media key ratchet, obtained from `Session.getKeyRatchet`.
#[napi]
pub struct KeyRatchet {
    pub(crate) inner: libdave::KeyRatchet,
}

/// Encrypts outgoing media frames.
#[napi]
pub struct Encryptor {
    inner: Option<libdave::Encryptor>,
}

impl Encryptor {
    fn get(&self) -> Result<&libdave::Encryptor> {
        self.inner.as_ref().ok_or_else(|| destroyed("Encryptor"))
    }

    fn get_mut(&mut self) -> Result<&mut libdave::Encryptor> {
        self.inner.as_mut().ok_or_else(|| destroyed("Encryptor"))
    }
}

#[napi]
impl Encryptor {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Some(libdave::Encryptor::new().map_err(to_napi)?),
        })
    }

    #[napi]
    pub fn set_key_ratchet(&mut self, ratchet: &KeyRatchet) -> Result<()> {
        self.get_mut()?.set_key_ratchet(&ratchet.inner);
        Ok(())
    }

    #[napi]
    pub fn set_passthrough_mode(&mut self, enabled: bool) -> Result<()> {
        self.get_mut()?.set_passthrough_mode(enabled);
        Ok(())
    }

    #[napi(getter)]
    pub fn passthrough_mode(&self) -> Result<bool> {
        Ok(self.get()?.is_passthrough_mode())
    }

    #[napi(getter)]
    pub fn has_key_ratchet(&self) -> Result<bool> {
        Ok(self.get()?.has_key_ratchet())
    }

    #[napi(getter)]
    pub fn protocol_version(&self) -> Result<u16> {
        Ok(self.get()?.protocol_version())
    }

    #[napi]
    pub fn assign_ssrc_to_codec(&mut self, ssrc: u32, codec: Codec) -> Result<()> {
        self.get_mut()?.assign_ssrc_to_codec(ssrc, codec.into());
        Ok(())
    }

    /// Encrypts one frame. Throws if libdave rejects it.
    #[napi]
    pub fn encrypt(&mut self, media_type: MediaType, ssrc: u32, frame: &[u8]) -> Result<Buffer> {
        self.get_mut()?
            .encrypt(media_type.into(), ssrc, frame)
            .map(Buffer::from)
            .map_err(to_napi)
    }

    #[napi]
    pub fn get_stats(&self, media_type: MediaType) -> Result<EncryptStats> {
        Ok(self.get()?.stats(media_type.into()).into())
    }

    /// Frees the native encryptor now instead of waiting for garbage collection.
    #[napi]
    pub fn destroy(&mut self) {
        self.inner = None;
    }
}

/// Decrypts media frames from one remote sender.
#[napi]
pub struct Decryptor {
    inner: Option<libdave::Decryptor>,
}

impl Decryptor {
    fn get_mut(&mut self) -> Result<&mut libdave::Decryptor> {
        self.inner.as_mut().ok_or_else(|| destroyed("Decryptor"))
    }
}

#[napi]
impl Decryptor {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Some(libdave::Decryptor::new().map_err(to_napi)?),
        })
    }

    #[napi]
    pub fn transition_to_key_ratchet(&mut self, ratchet: &KeyRatchet) -> Result<()> {
        self.get_mut()?.transition_to_key_ratchet(&ratchet.inner);
        Ok(())
    }

    #[napi]
    pub fn transition_to_passthrough_mode(&mut self, enabled: bool) -> Result<()> {
        self.get_mut()?.transition_to_passthrough_mode(enabled);
        Ok(())
    }

    /// Decrypts one frame. Throws if libdave rejects it.
    #[napi]
    pub fn decrypt(&mut self, media_type: MediaType, frame: &[u8]) -> Result<Buffer> {
        self.get_mut()?
            .decrypt(media_type.into(), frame)
            .map(Buffer::from)
            .map_err(to_napi)
    }

    #[napi]
    pub fn get_stats(&self, media_type: MediaType) -> Result<DecryptStats> {
        let inner = self.inner.as_ref().ok_or_else(|| destroyed("Decryptor"))?;
        Ok(inner.stats(media_type.into()).into())
    }

    /// Frees the native decryptor now instead of waiting for garbage collection.
    #[napi]
    pub fn destroy(&mut self) {
        self.inner = None;
    }
}
