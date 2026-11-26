use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::types::{Codec, DecryptStats, EncryptStats, Failures, MediaType, MlsFailure};
use crate::{destroyed, snowflake, to_napi};

#[napi(object)]
pub struct MlsCommitResult {
    pub ready: bool,
    pub failed: bool,
}

#[napi(object)]
pub struct MlsWelcomeResult {
    pub ready: bool,
}

/// Voice-gateway level DAVE bookkeeping: transitions, epochs, encryptor and decryptors.
#[napi]
pub struct DaveSessionManager {
    inner: Option<libdave_rs::SessionManager>,
    failures: Failures,
}

impl DaveSessionManager {
    fn get(&self) -> Result<&libdave_rs::SessionManager> {
        self.inner
            .as_ref()
            .ok_or_else(|| destroyed("DaveSessionManager"))
    }

    fn get_mut(&mut self) -> Result<&mut libdave_rs::SessionManager> {
        self.inner
            .as_mut()
            .ok_or_else(|| destroyed("DaveSessionManager"))
    }
}

#[napi]
impl DaveSessionManager {
    #[napi(constructor)]
    pub fn new(self_user_id: String, group_id: Either<String, BigInt>) -> Result<Self> {
        let group_id = snowflake(group_id, "groupId")?;
        let failures = Failures::default();
        let inner =
            libdave_rs::SessionManager::new(self_user_id, group_id, Some(failures.handler()))
                .map_err(to_napi)?;
        Ok(Self {
            inner: Some(inner),
            failures,
        })
    }

    #[napi(getter)]
    pub fn protocol_version(&self) -> Result<u16> {
        Ok(self.get()?.protocol_version())
    }

    #[napi]
    pub fn add_user(&mut self, user_id: String) -> Result<()> {
        self.get_mut()?.add_user(&user_id).map_err(to_napi)
    }

    #[napi]
    pub fn remove_user(&mut self, user_id: String) -> Result<()> {
        self.get_mut()?.remove_user(&user_id).map_err(to_napi)
    }

    #[napi]
    pub fn on_select_protocol_ack(&mut self, protocol_version: u16) -> Result<()> {
        self.get_mut()?
            .on_select_protocol_ack(protocol_version)
            .map_err(to_napi)
    }

    /// Returns the transition id to acknowledge, or `null` for transition `0`.
    #[napi]
    pub fn on_prepare_transition(
        &mut self,
        transition_id: u16,
        protocol_version: u16,
    ) -> Result<Option<u16>> {
        self.get_mut()?
            .on_prepare_transition(transition_id, protocol_version)
            .map_err(to_napi)
    }

    #[napi]
    pub fn on_execute_transition(&mut self, transition_id: u16) -> Result<()> {
        self.get_mut()?
            .on_execute_transition(transition_id)
            .map_err(to_napi)
    }

    #[napi]
    pub fn on_prepare_epoch(
        &mut self,
        epoch: Either<u32, BigInt>,
        protocol_version: u16,
    ) -> Result<()> {
        let epoch = match epoch {
            Either::A(number) => u64::from(number),
            Either::B(big) => snowflake(Either::B(big), "epoch")?,
        };
        self.get_mut()?
            .on_prepare_epoch(epoch, protocol_version)
            .map_err(to_napi)
    }

    #[napi]
    pub fn on_external_sender_package(&mut self, data: &[u8]) -> Result<()> {
        self.get_mut()?.on_external_sender_package(data);
        Ok(())
    }

    #[napi]
    pub fn on_mls_proposals(&mut self, proposals: &[u8]) -> Result<Option<Buffer>> {
        Ok(self
            .get_mut()?
            .on_mls_proposals(proposals)
            .map(Buffer::from))
    }

    #[napi]
    pub fn on_mls_commit(&mut self, transition_id: u16, commit: &[u8]) -> Result<MlsCommitResult> {
        let result = self
            .get_mut()?
            .on_mls_commit(transition_id, commit)
            .map_err(to_napi)?;
        Ok(MlsCommitResult {
            ready: result.ready,
            failed: result.failed,
        })
    }

    #[napi]
    pub fn on_mls_welcome(
        &mut self,
        transition_id: u16,
        welcome: &[u8],
    ) -> Result<MlsWelcomeResult> {
        let result = self
            .get_mut()?
            .on_mls_welcome(transition_id, welcome)
            .map_err(to_napi)?;
        Ok(MlsWelcomeResult {
            ready: result.ready,
        })
    }

    #[napi]
    pub fn get_marshalled_key_package(&self) -> Result<Buffer> {
        Ok(self.get()?.marshalled_key_package().into())
    }

    /// Encrypts an outgoing frame, or returns `null` if it cannot be encrypted.
    #[napi]
    pub fn encrypt(
        &mut self,
        media_type: MediaType,
        ssrc: u32,
        frame: &[u8],
    ) -> Result<Option<Buffer>> {
        Ok(self
            .get_mut()?
            .encrypt(media_type.into(), ssrc, frame)
            .ok()
            .map(Buffer::from))
    }

    /// Decrypts a frame from `userId`, or returns `null` if it cannot be decrypted.
    #[napi]
    pub fn decrypt(
        &mut self,
        media_type: MediaType,
        user_id: String,
        frame: &[u8],
    ) -> Result<Option<Buffer>> {
        Ok(self
            .get_mut()?
            .decrypt(&user_id, media_type.into(), frame)
            .ok()
            .map(Buffer::from))
    }

    #[napi]
    pub fn assign_ssrc_codec(&mut self, ssrc: u32, codec: Codec) -> Result<()> {
        self.get_mut()?
            .encryptor_mut()
            .assign_ssrc_to_codec(ssrc, codec.into());
        Ok(())
    }

    #[napi]
    pub fn get_encrypt_stats(&mut self, media_type: MediaType) -> Result<EncryptStats> {
        Ok(self
            .get_mut()?
            .encryptor_mut()
            .stats(media_type.into())
            .into())
    }

    /// Returns `null` if nothing has been set up for `userId`.
    #[napi]
    pub fn get_decrypt_stats(
        &self,
        user_id: String,
        media_type: MediaType,
    ) -> Result<Option<DecryptStats>> {
        Ok(self
            .get()?
            .decryptor_stats(&user_id, media_type.into())
            .map(DecryptStats::from))
    }

    /// Returns and clears the MLS failures reported since the last call.
    #[napi]
    pub fn take_failures(&mut self) -> Vec<MlsFailure> {
        self.failures.take()
    }

    #[napi]
    pub fn destroy(&mut self) {
        self.inner = None;
    }
}
