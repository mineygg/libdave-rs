use std::sync::{Arc, Mutex};

use napi_derive::napi;

/// Media type of a frame.
#[napi]
pub enum MediaType {
    Audio = 0,
    Video = 1,
}

impl From<MediaType> for libdave::MediaType {
    fn from(value: MediaType) -> Self {
        match value {
            MediaType::Audio => Self::Audio,
            MediaType::Video => Self::Video,
        }
    }
}

/// Codec carried by an SSRC.
#[napi]
pub enum Codec {
    Unknown = 0,
    Opus = 1,
    Vp8 = 2,
    Vp9 = 3,
    H264 = 4,
    H265 = 5,
    Av1 = 6,
}

impl From<Codec> for libdave::Codec {
    fn from(value: Codec) -> Self {
        match value {
            Codec::Unknown => Self::Unknown,
            Codec::Opus => Self::Opus,
            Codec::Vp8 => Self::Vp8,
            Codec::Vp9 => Self::Vp9,
            Codec::H264 => Self::H264,
            Codec::H265 => Self::H265,
            Codec::Av1 => Self::Av1,
        }
    }
}

/// Counters reported by an encryptor.
#[napi(object)]
pub struct EncryptStats {
    pub success: f64,
    pub failure: f64,
    pub passthrough: f64,
    pub missing_key: f64,
    pub attempts: f64,
    pub max_attempts: f64,
    pub duration: f64,
}

impl From<libdave::EncryptorStats> for EncryptStats {
    fn from(s: libdave::EncryptorStats) -> Self {
        Self {
            success: s.encryptSuccessCount as f64,
            failure: s.encryptFailureCount as f64,
            passthrough: s.passthroughCount as f64,
            missing_key: s.encryptMissingKeyCount as f64,
            attempts: s.encryptAttempts as f64,
            max_attempts: s.encryptMaxAttempts as f64,
            duration: s.encryptDuration as f64,
        }
    }
}

/// Counters reported by a decryptor.
#[napi(object)]
pub struct DecryptStats {
    pub success: f64,
    pub failure: f64,
    pub passthrough: f64,
    pub missing_key: f64,
    pub invalid_nonce: f64,
    pub attempts: f64,
    pub duration: f64,
}

impl From<libdave::DecryptorStats> for DecryptStats {
    fn from(s: libdave::DecryptorStats) -> Self {
        Self {
            success: s.decryptSuccessCount as f64,
            failure: s.decryptFailureCount as f64,
            passthrough: s.passthroughCount as f64,
            missing_key: s.decryptMissingKeyCount as f64,
            invalid_nonce: s.decryptInvalidNonceCount as f64,
            attempts: s.decryptAttempts as f64,
            duration: s.decryptDuration as f64,
        }
    }
}

/// An MLS failure reported by libdave.
#[napi(object)]
pub struct MlsFailure {
    pub source: String,
    pub reason: String,
}

/// Failures are queued here and drained from JavaScript with `takeFailures()`.
#[derive(Clone, Default)]
pub(crate) struct Failures(Arc<Mutex<Vec<MlsFailure>>>);

impl Failures {
    const LIMIT: usize = 64;

    pub(crate) fn handler(&self) -> libdave::FailureHandler {
        let queue = self.clone();
        Box::new(move |source, reason| {
            if let Ok(mut items) = queue.0.lock() {
                if items.len() == Self::LIMIT {
                    items.remove(0);
                }
                items.push(MlsFailure {
                    source: source.to_owned(),
                    reason: reason.to_owned(),
                });
            }
        })
    }

    pub(crate) fn take(&self) -> Vec<MlsFailure> {
        self.0
            .lock()
            .map(|mut items| std::mem::take(&mut *items))
            .unwrap_or_default()
    }
}
