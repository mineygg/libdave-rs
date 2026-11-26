//! Safe Rust bindings to Discord's DAVE (end-to-end encrypted audio/video) implementation,
//! [libdave](https://github.com/discord/libdave).
//!
//! * [`Session`], [`Encryptor`], [`Decryptor`] and [`KeyRatchet`] are thin RAII wrappers over the
//!   C API.
//! * [`SessionManager`] implements the voice-gateway transition / epoch bookkeeping on top of
//!   them.
//!
//! libdave, mlspp and OpenSSL are compiled from source and linked statically, so no system
//! libraries are required.

mod crypto;
mod error;
mod manager;
mod session;
mod util;

use std::os::raw::{c_char, c_int};
use std::sync::RwLock;

use libdave_rs_sys as sys;

pub use crypto::{Decryptor, DecryptorStats, Encryptor, EncryptorStats, KeyRatchet};
pub use error::{DecryptError, EncryptError, Error, Result};
pub use manager::{CommitResult, SessionManager, WelcomeResult};
pub use session::{CommitOutcome, FailureHandler, Session, WelcomeOutcome};

/// Media type of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MediaType {
    Audio = sys::DAVE_MEDIA_TYPE_AUDIO,
    Video = sys::DAVE_MEDIA_TYPE_VIDEO,
}

/// Codec carried by an SSRC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Codec {
    Unknown = sys::DAVE_CODEC_UNKNOWN,
    Opus = sys::DAVE_CODEC_OPUS,
    Vp8 = sys::DAVE_CODEC_VP8,
    Vp9 = sys::DAVE_CODEC_VP9,
    H264 = sys::DAVE_CODEC_H264,
    H265 = sys::DAVE_CODEC_H265,
    Av1 = sys::DAVE_CODEC_AV1,
}

/// Highest DAVE protocol version supported by the linked libdave.
pub fn max_supported_protocol_version() -> u16 {
    unsafe { sys::daveMaxSupportedProtocolVersion() }
}

/// Severity of a libdave log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogSeverity {
    Verbose,
    Info,
    Warning,
    Error,
}

/// Process-wide log sink: `(severity, file, line, message)`.
pub type LogSink = Box<dyn Fn(LogSeverity, &str, i32, &str) + Send + Sync>;
static LOG_SINK: RwLock<Option<LogSink>> = RwLock::new(None);

unsafe extern "C" fn log_trampoline(
    severity: u32,
    file: *const c_char,
    line: c_int,
    message: *const c_char,
) {
    let level = match severity {
        sys::DAVE_LOGGING_SEVERITY_VERBOSE => LogSeverity::Verbose,
        sys::DAVE_LOGGING_SEVERITY_INFO => LogSeverity::Info,
        sys::DAVE_LOGGING_SEVERITY_WARNING => LogSeverity::Warning,
        sys::DAVE_LOGGING_SEVERITY_ERROR => LogSeverity::Error,
        _ => return,
    };
    let (file, message) = unsafe { (util::cstr_lossy(file), util::cstr_lossy(message)) };
    if let Ok(guard) = LOG_SINK.read() {
        if let Some(sink) = guard.as_ref() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                sink(level, &file, line, &message)
            }));
        }
    }
}

/// Installs (or removes, with `None`) a process-wide sink for libdave's log output.
pub fn set_log_sink(sink: Option<LogSink>) {
    let install = sink.is_some();
    if let Ok(mut guard) = LOG_SINK.write() {
        *guard = sink;
    }
    unsafe {
        sys::daveSetLogSinkCallback(if install { Some(log_trampoline) } else { None });
    }
}
