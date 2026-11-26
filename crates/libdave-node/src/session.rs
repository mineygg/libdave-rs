use std::sync::mpsc::Receiver;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::crypto::KeyRatchet;
use crate::types::{Failures, MlsFailure};
use crate::{destroyed, snowflake, to_napi};

#[napi(object)]
pub struct CommitOutcome {
    pub failed: bool,
    pub ignored: bool,
    /// User ids in the roster after the commit.
    pub roster: Vec<String>,
}

#[napi(object)]
pub struct WelcomeOutcome {
    /// User ids in the roster of the group that was joined.
    pub roster: Vec<String>,
}

fn ids(roster: Vec<u64>) -> Vec<String> {
    roster.into_iter().map(|id| id.to_string()).collect()
}

pub struct FingerprintTask(Receiver<Vec<u8>>);

impl Task for FingerprintTask {
    type Output = Vec<u8>;
    type JsValue = Buffer;

    fn compute(&mut self) -> Result<Self::Output> {
        match self.0.recv() {
            Ok(bytes) if !bytes.is_empty() => Ok(bytes),
            _ => Err(Error::from_reason(
                "could not compute the pairwise fingerprint (no MLS group or unknown user)",
            )),
        }
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output.into())
    }
}

/// A DAVE MLS session.
#[napi]
pub struct Session {
    inner: Option<libdave::Session>,
    failures: Failures,
}

impl Session {
    fn get(&self) -> Result<&libdave::Session> {
        self.inner.as_ref().ok_or_else(|| destroyed("Session"))
    }

    fn get_mut(&mut self) -> Result<&mut libdave::Session> {
        self.inner.as_mut().ok_or_else(|| destroyed("Session"))
    }
}

#[napi]
impl Session {
    #[napi(constructor)]
    pub fn new(auth_session_id: Option<String>) -> Result<Self> {
        let failures = Failures::default();
        let inner = libdave::Session::new(auth_session_id.as_deref(), Some(failures.handler()))
            .map_err(to_napi)?;
        Ok(Self {
            inner: Some(inner),
            failures,
        })
    }

    /// Initializes the session for `groupId` (the voice channel id) as `selfUserId`.
    #[napi]
    pub fn init(
        &mut self,
        version: u16,
        group_id: Either<String, BigInt>,
        self_user_id: String,
    ) -> Result<()> {
        let group_id = snowflake(group_id, "groupId")?;
        self.get_mut()?
            .init(version, group_id, &self_user_id)
            .map_err(to_napi)
    }

    #[napi]
    pub fn reset(&mut self) -> Result<()> {
        self.get_mut()?.reset();
        Ok(())
    }

    #[napi]
    pub fn set_protocol_version(&mut self, version: u16) -> Result<()> {
        self.get_mut()?.set_protocol_version(version);
        Ok(())
    }

    #[napi(getter)]
    pub fn protocol_version(&self) -> Result<u16> {
        Ok(self.get()?.protocol_version())
    }

    #[napi]
    pub fn set_external_sender(&mut self, external_sender: &[u8]) -> Result<()> {
        self.get_mut()?.set_external_sender(external_sender);
        Ok(())
    }

    /// Marks a user as connected to the call. MLS messages naming other users are rejected.
    #[napi]
    pub fn add_recognized_user(&mut self, user_id: String) -> Result<()> {
        self.get_mut()?
            .add_recognized_user(&user_id)
            .map_err(to_napi)
    }

    #[napi]
    pub fn remove_recognized_user(&mut self, user_id: String) -> Result<()> {
        self.get_mut()?
            .remove_recognized_user(&user_id)
            .map_err(to_napi)
    }

    /// Returns the commit/welcome bytes to send back, or `null` if none were produced.
    #[napi]
    pub fn process_proposals(&mut self, proposals: &[u8]) -> Result<Option<Buffer>> {
        Ok(self
            .get_mut()?
            .process_proposals(proposals)
            .map(Buffer::from))
    }

    #[napi]
    pub fn process_commit(&mut self, commit: &[u8]) -> Result<CommitOutcome> {
        let outcome = self.get_mut()?.process_commit(commit);
        Ok(CommitOutcome {
            failed: outcome.failed,
            ignored: outcome.ignored,
            roster: ids(outcome.roster),
        })
    }

    /// Returns `null` if the welcome could not be processed.
    #[napi]
    pub fn process_welcome(&mut self, welcome: &[u8]) -> Result<Option<WelcomeOutcome>> {
        Ok(self
            .get_mut()?
            .process_welcome(welcome)
            .map(|outcome| WelcomeOutcome {
                roster: ids(outcome.roster),
            }))
    }

    #[napi]
    pub fn get_marshalled_key_package(&self) -> Result<Buffer> {
        Ok(self.get()?.marshalled_key_package().into())
    }

    #[napi]
    pub fn get_key_ratchet(&self, user_id: String) -> Result<Option<KeyRatchet>> {
        Ok(self
            .get()?
            .key_ratchet(&user_id)
            .map_err(to_napi)?
            .map(|inner| KeyRatchet { inner }))
    }

    #[napi]
    pub fn get_last_epoch_authenticator(&self) -> Result<Buffer> {
        Ok(self.get()?.last_epoch_authenticator().into())
    }

    /// Derives the identity fingerprint shared with `userId` off the main thread.
    #[napi(ts_return_type = "Promise<Buffer>")]
    pub fn get_pairwise_fingerprint(
        &self,
        version: u16,
        user_id: String,
    ) -> Result<AsyncTask<FingerprintTask>> {
        let receiver = self
            .get()?
            .request_pairwise_fingerprint(version, &user_id)
            .map_err(to_napi)?;
        Ok(AsyncTask::new(FingerprintTask(receiver)))
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
