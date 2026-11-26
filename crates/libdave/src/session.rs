use std::os::raw::{c_char, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::mpsc::{channel, Receiver, Sender};

use libdave_sys as sys;

use crate::crypto::KeyRatchet;
use crate::error::{Error, Result};
use crate::util::{cstr_lossy, cstring, take_bytes, take_u64s, CStringArray};

/// Callback invoked as `(source, reason)` when libdave reports an MLS failure.
pub type FailureHandler = Box<dyn FnMut(&str, &str) + Send>;

unsafe extern "C" fn failure_trampoline(
    source: *const c_char,
    reason: *const c_char,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    // SAFETY: `user_data` is the `Box<FailureHandler>` owned by the `Session`.
    let cb = unsafe { &mut *user_data.cast::<FailureHandler>() };
    let (source, reason) = unsafe { (cstr_lossy(source), cstr_lossy(reason)) };
    let _ = catch_unwind(AssertUnwindSafe(|| cb(&source, &reason)));
}

/// Outcome of [`Session::process_commit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitOutcome {
    pub failed: bool,
    pub ignored: bool,
    /// User ids (snowflakes) in the roster after the commit.
    pub roster: Vec<u64>,
}

/// Outcome of [`Session::process_welcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WelcomeOutcome {
    /// User ids (snowflakes) in the roster of the group that was joined.
    pub roster: Vec<u64>,
}

/// A DAVE MLS session (`DAVESessionHandle`).
pub struct Session {
    raw: sys::DAVESessionHandle,
    callback: *mut FailureHandler,
}

// SAFETY: libdave sessions have no thread affinity; all methods take `&mut self`/`&self`
// and the failure callback is required to be `Send`.
unsafe impl Send for Session {}

impl Session {
    /// Creates a new session.
    ///
    /// `auth_session_id` scopes persistent key lifetimes (only relevant with the
    /// `persistent-keys` feature). `on_failure` is invoked as `(source, reason)` whenever
    /// libdave reports an MLS failure.
    pub fn new(auth_session_id: Option<&str>, on_failure: Option<FailureHandler>) -> Result<Self> {
        let auth = auth_session_id
            .map(|s| cstring(s, "auth session id"))
            .transpose()?;
        let auth_ptr = auth.as_ref().map_or(ptr::null(), |c| c.as_ptr());

        let (cb_ptr, cb_fn): (*mut FailureHandler, Option<sys::DAVEMLSFailureCallback>) =
            match on_failure {
                Some(f) => {
                    let boxed: Box<FailureHandler> = Box::new(f);
                    (Box::into_raw(boxed), Some(failure_trampoline))
                }
                None => (ptr::null_mut(), None),
            };

        let raw =
            unsafe { sys::daveSessionCreate(ptr::null_mut(), auth_ptr, cb_fn, cb_ptr.cast()) };
        if raw.is_null() {
            if !cb_ptr.is_null() {
                drop(unsafe { Box::from_raw(cb_ptr) });
            }
            return Err(Error::CreateFailed("session"));
        }
        Ok(Self {
            raw,
            callback: cb_ptr,
        })
    }

    /// Initializes the session for `group_id` (the voice channel id) as `self_user_id`.
    pub fn init(&mut self, version: u16, group_id: u64, self_user_id: &str) -> Result<()> {
        let id = cstring(self_user_id, "self user id")?;
        unsafe { sys::daveSessionInit(self.raw, version, group_id, id.as_ptr()) };
        Ok(())
    }

    /// Resets all MLS state.
    pub fn reset(&mut self) {
        unsafe { sys::daveSessionReset(self.raw) };
    }

    pub fn set_protocol_version(&mut self, version: u16) {
        unsafe { sys::daveSessionSetProtocolVersion(self.raw, version) };
    }

    pub fn protocol_version(&self) -> u16 {
        unsafe { sys::daveSessionGetProtocolVersion(self.raw) }
    }

    /// Feeds the voice gateway's `DAVE_MLS_EXTERNAL_SENDER_PACKAGE` payload.
    pub fn set_external_sender(&mut self, external_sender: &[u8]) {
        unsafe {
            sys::daveSessionSetExternalSender(
                self.raw,
                external_sender.as_ptr(),
                external_sender.len(),
            )
        };
    }

    /// Processes `DAVE_MLS_PROPOSALS`. Returns the serialized commit (+ welcome) to send back,
    /// if one was produced.
    pub fn process_proposals<S: AsRef<str>>(
        &mut self,
        proposals: &[u8],
        recognized_user_ids: &[S],
    ) -> Result<Option<Vec<u8>>> {
        let ids = CStringArray::new(recognized_user_ids)?;
        let mut out: *mut u8 = ptr::null_mut();
        let mut len = 0usize;
        unsafe {
            sys::daveSessionProcessProposals(
                self.raw,
                proposals.as_ptr(),
                proposals.len(),
                ids.as_ptr(),
                ids.len(),
                &mut out,
                &mut len,
            );
            Ok(take_bytes(out, len).filter(|v| !v.is_empty()))
        }
    }

    /// Processes `DAVE_MLS_ANNOUNCE_COMMIT_TRANSITION`.
    pub fn process_commit(&mut self, commit: &[u8]) -> CommitOutcome {
        let handle =
            unsafe { sys::daveSessionProcessCommit(self.raw, commit.as_ptr(), commit.len()) };
        if handle.is_null() {
            return CommitOutcome {
                failed: true,
                ignored: false,
                roster: Vec::new(),
            };
        }
        let failed = unsafe { sys::daveCommitResultIsFailed(handle) };
        let ignored = unsafe { sys::daveCommitResultIsIgnored(handle) };
        let mut roster = Vec::new();
        if !failed && !ignored {
            let mut ids: *mut u64 = ptr::null_mut();
            let mut len = 0usize;
            unsafe {
                sys::daveCommitResultGetRosterMemberIds(handle, &mut ids, &mut len);
                roster = take_u64s(ids, len);
            }
        }
        unsafe { sys::daveCommitResultDestroy(handle) };
        CommitOutcome {
            failed,
            ignored,
            roster,
        }
    }

    /// Processes `DAVE_MLS_WELCOME`. Returns `None` if the welcome could not be processed.
    pub fn process_welcome<S: AsRef<str>>(
        &mut self,
        welcome: &[u8],
        recognized_user_ids: &[S],
    ) -> Result<Option<WelcomeOutcome>> {
        let ids = CStringArray::new(recognized_user_ids)?;
        let handle = unsafe {
            sys::daveSessionProcessWelcome(
                self.raw,
                welcome.as_ptr(),
                welcome.len(),
                ids.as_ptr(),
                ids.len(),
            )
        };
        if handle.is_null() {
            return Ok(None);
        }
        let mut out: *mut u64 = ptr::null_mut();
        let mut len = 0usize;
        let roster = unsafe {
            sys::daveWelcomeResultGetRosterMemberIds(handle, &mut out, &mut len);
            take_u64s(out, len)
        };
        unsafe { sys::daveWelcomeResultDestroy(handle) };
        Ok(Some(WelcomeOutcome { roster }))
    }

    /// The marshalled MLS key package to send in `DAVE_MLS_KEY_PACKAGE`.
    pub fn marshalled_key_package(&self) -> Vec<u8> {
        let mut out: *mut u8 = ptr::null_mut();
        let mut len = 0usize;
        unsafe {
            sys::daveSessionGetMarshalledKeyPackage(self.raw, &mut out, &mut len);
            take_bytes(out, len).unwrap_or_default()
        }
    }

    /// Returns the media key ratchet for `user_id`, if the session has one.
    pub fn key_ratchet(&self, user_id: &str) -> Result<Option<KeyRatchet>> {
        let id = cstring(user_id, "user id")?;
        let raw = unsafe { sys::daveSessionGetKeyRatchet(self.raw, id.as_ptr()) };
        Ok(KeyRatchet::from_raw(raw))
    }

    /// Authenticator of the last MLS epoch (for out-of-band verification).
    pub fn last_epoch_authenticator(&self) -> Vec<u8> {
        let mut out: *mut u8 = ptr::null_mut();
        let mut len = 0usize;
        unsafe {
            sys::daveSessionGetLastEpochAuthenticator(self.raw, &mut out, &mut len);
            take_bytes(out, len).unwrap_or_default()
        }
    }

    /// Starts computing the pairwise identity fingerprint with `user_id`.
    ///
    /// libdave derives it on a background thread; the returned channel yields it exactly once.
    /// An empty vector means it could not be computed (no group yet, or unknown user).
    pub fn request_pairwise_fingerprint(
        &self,
        version: u16,
        user_id: &str,
    ) -> Result<Receiver<Vec<u8>>> {
        unsafe extern "C" fn deliver(fp: *const u8, len: usize, user_data: *mut c_void) {
            // SAFETY: `user_data` is the boxed sender below; libdave calls us exactly once.
            let tx = unsafe { Box::from_raw(user_data.cast::<Sender<Vec<u8>>>()) };
            let bytes = if fp.is_null() || len == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(fp, len) }.to_vec()
            };
            let _ = tx.send(bytes);
        }
        let id = cstring(user_id, "user id")?;
        let (tx, rx) = channel::<Vec<u8>>();
        unsafe {
            sys::daveSessionGetPairwiseFingerprint(
                self.raw,
                version,
                id.as_ptr(),
                deliver,
                Box::into_raw(Box::new(tx)).cast(),
            )
        };
        Ok(rx)
    }

    /// Blocking convenience wrapper around [`Session::request_pairwise_fingerprint`].
    pub fn pairwise_fingerprint(&self, version: u16, user_id: &str) -> Result<Vec<u8>> {
        Ok(self
            .request_pairwise_fingerprint(version, user_id)?
            .recv()
            .unwrap_or_default())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { sys::daveSessionDestroy(self.raw) };
        if !self.callback.is_null() {
            drop(unsafe { Box::from_raw(self.callback) });
        }
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("protocol_version", &self.protocol_version())
            .finish()
    }
}
