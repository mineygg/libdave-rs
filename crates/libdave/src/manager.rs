//! Voice-gateway DAVE bookkeeping: protocol transitions, epochs and per-user ratchets.
//!
//! This is a Rust port of the `DaveSessionManager` used by cherrycord's `vocalize` package,
//! without the Node-API layer. It owns one [`Session`], the local [`Encryptor`] and one
//! [`Decryptor`] per remote user, and keeps them in sync with the opcodes the voice gateway
//! sends (`SELECT_PROTOCOL_ACK`, `DAVE_PREPARE_TRANSITION`, `DAVE_EXECUTE_TRANSITION`,
//! `DAVE_PREPARE_EPOCH`, MLS proposals/commits/welcomes, ...).

use std::collections::{HashMap, HashSet};

use crate::crypto::{Decryptor, Encryptor};
use crate::error::Result;
use crate::session::Session;
use crate::MediaType;

/// Result of [`SessionManager::on_mls_commit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitResult {
    /// The transition is prepared; send `DAVE_TRANSITION_READY` for the transition id.
    pub ready: bool,
    /// libdave rejected the commit; recover by re-sending `DAVE_MLS_INVALID_COMMIT_WELCOME`.
    pub failed: bool,
}

/// Result of [`SessionManager::on_mls_welcome`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WelcomeResult {
    /// The transition is prepared; send `DAVE_TRANSITION_READY` for the transition id.
    pub ready: bool,
}

pub struct SessionManager {
    session: Session,
    self_user_id: String,
    group_id: u64,
    recognized_users: HashSet<String>,
    encryptor: Encryptor,
    decryptors: HashMap<String, Decryptor>,
    pending_transitions: HashMap<u16, u16>,
    latest_version: u16,
}

impl SessionManager {
    /// Creates a manager for `self_user_id` in the group (voice channel) `group_id`.
    pub fn new(
        self_user_id: impl Into<String>,
        group_id: u64,
        on_failure: Option<Box<dyn FnMut(&str, &str) + Send>>,
    ) -> Result<Self> {
        Ok(Self {
            session: Session::new(None, on_failure)?,
            self_user_id: self_user_id.into(),
            group_id,
            recognized_users: HashSet::new(),
            encryptor: Encryptor::new()?,
            decryptors: HashMap::new(),
            pending_transitions: HashMap::new(),
            latest_version: 0,
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn encryptor_mut(&mut self) -> &mut Encryptor {
        &mut self.encryptor
    }

    pub fn protocol_version(&self) -> u16 {
        self.latest_version
    }

    fn decryptor(&mut self, user_id: &str) -> Result<&mut Decryptor> {
        if !self.decryptors.contains_key(user_id) {
            self.decryptors
                .insert(user_id.to_owned(), Decryptor::new()?);
        }
        Ok(self.decryptors.get_mut(user_id).expect("just inserted"))
    }

    fn refresh_ratchet(&mut self, user_id: &str, version: u16) -> Result<()> {
        let is_self = user_id == self.self_user_id;

        if version == 0 {
            if is_self {
                self.encryptor.set_passthrough_mode(true);
            } else {
                self.decryptor(user_id)?
                    .transition_to_passthrough_mode(true);
            }
            return Ok(());
        }

        let Some(ratchet) = self.session.key_ratchet(user_id)? else {
            return Ok(());
        };
        if is_self {
            self.encryptor.set_passthrough_mode(false);
            self.encryptor.set_key_ratchet(&ratchet);
        } else {
            let dec = self.decryptor(user_id)?;
            dec.transition_to_key_ratchet(&ratchet);
            // Let the passthrough grace period expire now that real keys exist.
            dec.transition_to_passthrough_mode(false);
        }
        Ok(())
    }

    fn prepare_ratchets(&mut self, transition_id: u16, version: u16) -> Result<()> {
        let others: Vec<String> = self
            .recognized_users
            .iter()
            .filter(|u| **u != self.self_user_id)
            .cloned()
            .collect();
        for user in others {
            self.refresh_ratchet(&user, version)?;
        }
        if transition_id == 0 {
            let me = self.self_user_id.clone();
            self.refresh_ratchet(&me, version)?;
        } else {
            self.pending_transitions.insert(transition_id, version);
        }
        self.latest_version = version;
        Ok(())
    }

    fn handle_protocol_init(&mut self, version: u16) -> Result<()> {
        if version > 0 {
            self.session
                .init(version, self.group_id, &self.self_user_id)?;
        } else {
            self.prepare_ratchets(0, version)?;
            let me = self.self_user_id.clone();
            self.refresh_ratchet(&me, version)?;
        }
        Ok(())
    }

    fn all_recognized_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.recognized_users.iter().cloned().collect();
        ids.push(self.self_user_id.clone());
        ids
    }

    /// A user joined the call.
    pub fn add_user(&mut self, user_id: &str) -> Result<()> {
        self.recognized_users.insert(user_id.to_owned());
        let version = self.latest_version;
        self.refresh_ratchet(user_id, version)
    }

    /// A user left the call.
    pub fn remove_user(&mut self, user_id: &str) {
        self.recognized_users.remove(user_id);
        self.decryptors.remove(user_id);
    }

    /// `SELECT_PROTOCOL_ACK` carried `dave_protocol_version`.
    pub fn on_select_protocol_ack(&mut self, version: u16) -> Result<()> {
        self.handle_protocol_init(version)
    }

    /// `DAVE_PREPARE_TRANSITION`. Returns the transition id to acknowledge with
    /// `DAVE_TRANSITION_READY`, or `None` for the implicit transition `0`.
    pub fn on_prepare_transition(
        &mut self,
        transition_id: u16,
        version: u16,
    ) -> Result<Option<u16>> {
        self.prepare_ratchets(transition_id, version)?;
        Ok((transition_id != 0).then_some(transition_id))
    }

    /// `DAVE_EXECUTE_TRANSITION`.
    pub fn on_execute_transition(&mut self, transition_id: u16) -> Result<()> {
        if let Some(version) = self.pending_transitions.remove(&transition_id) {
            if version == 0 {
                self.session.reset();
            }
            let me = self.self_user_id.clone();
            self.refresh_ratchet(&me, version)?;
        }
        Ok(())
    }

    /// `DAVE_PREPARE_EPOCH`. Epoch `1` starts a brand new MLS group.
    pub fn on_prepare_epoch(&mut self, epoch: u64, version: u16) -> Result<()> {
        if epoch == 1 {
            self.session
                .init(version, self.group_id, &self.self_user_id)?;
        }
        Ok(())
    }

    /// `DAVE_MLS_EXTERNAL_SENDER_PACKAGE`.
    pub fn on_external_sender_package(&mut self, data: &[u8]) {
        self.session.set_external_sender(data);
    }

    /// `DAVE_MLS_PROPOSALS`. Returns the commit/welcome bytes to send back, if any.
    /// `user_ids` defaults to every recognized user plus ourselves.
    pub fn on_mls_proposals(
        &mut self,
        proposals: &[u8],
        user_ids: Option<&[String]>,
    ) -> Result<Option<Vec<u8>>> {
        let ids = user_ids.map_or_else(|| self.all_recognized_ids(), <[String]>::to_vec);
        self.session.process_proposals(proposals, &ids)
    }

    /// `DAVE_MLS_ANNOUNCE_COMMIT_TRANSITION`.
    pub fn on_mls_commit(&mut self, transition_id: u16, commit: &[u8]) -> Result<CommitResult> {
        let outcome = self.session.process_commit(commit);
        let mut ready = false;
        if !outcome.failed && !outcome.ignored {
            let version = self.session.protocol_version();
            if outcome.roster.is_empty() {
                self.handle_protocol_init(version)?;
            } else {
                self.prepare_ratchets(transition_id, version)?;
                ready = transition_id != 0;
            }
        }
        Ok(CommitResult {
            ready,
            failed: outcome.failed,
        })
    }

    /// `DAVE_MLS_WELCOME`.
    pub fn on_mls_welcome(
        &mut self,
        transition_id: u16,
        welcome: &[u8],
        user_ids: Option<&[String]>,
    ) -> Result<WelcomeResult> {
        let ids = user_ids.map_or_else(|| self.all_recognized_ids(), <[String]>::to_vec);
        let mut ready = false;
        if let Some(outcome) = self.session.process_welcome(welcome, &ids)? {
            if !outcome.roster.is_empty() {
                let version = self.session.protocol_version();
                self.prepare_ratchets(transition_id, version)?;
                ready = transition_id != 0;
            }
        }
        Ok(WelcomeResult { ready })
    }

    /// The key package to send in `DAVE_MLS_KEY_PACKAGE`.
    pub fn marshalled_key_package(&self) -> Vec<u8> {
        self.session.marshalled_key_package()
    }

    /// Encrypts an outgoing frame with the local encryptor.
    pub fn encrypt(&mut self, media: MediaType, ssrc: u32, frame: &[u8]) -> Result<Vec<u8>> {
        self.encryptor.encrypt(media, ssrc, frame)
    }

    /// Decrypts a frame received from `user_id`.
    pub fn decrypt(&mut self, user_id: &str, media: MediaType, frame: &[u8]) -> Result<Vec<u8>> {
        self.decryptor(user_id)?.decrypt(media, frame)
    }
}
