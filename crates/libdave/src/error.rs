use std::fmt;

/// Errors returned by this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// libdave failed to allocate / create an object.
    CreateFailed(&'static str),
    /// An argument could not be passed to libdave (e.g. a user id containing a NUL byte).
    InvalidArgument(&'static str),
    /// Encryption failed with the given libdave result code.
    Encrypt(EncryptError),
    /// Decryption failed with the given libdave result code.
    Decrypt(DecryptError),
}

/// Result codes of `daveEncryptorEncrypt` other than success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptError {
    EncryptionFailure,
    MissingKeyRatchet,
    MissingCryptor,
    TooManyAttempts,
    Unknown(u32),
}

/// Result codes of `daveDecryptorDecrypt` other than success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecryptError {
    DecryptionFailure,
    MissingKeyRatchet,
    InvalidNonce,
    MissingCryptor,
    Unknown(u32),
}

impl EncryptError {
    pub(crate) fn from_code(code: u32) -> Self {
        use libdave_rs_sys as s;
        match code {
            s::DAVE_ENCRYPTOR_RESULT_CODE_ENCRYPTION_FAILURE => Self::EncryptionFailure,
            s::DAVE_ENCRYPTOR_RESULT_CODE_MISSING_KEY_RATCHET => Self::MissingKeyRatchet,
            s::DAVE_ENCRYPTOR_RESULT_CODE_MISSING_CRYPTOR => Self::MissingCryptor,
            s::DAVE_ENCRYPTOR_RESULT_CODE_TOO_MANY_ATTEMPTS => Self::TooManyAttempts,
            other => Self::Unknown(other),
        }
    }
}

impl DecryptError {
    pub(crate) fn from_code(code: u32) -> Self {
        use libdave_rs_sys as s;
        match code {
            s::DAVE_DECRYPTOR_RESULT_CODE_DECRYPTION_FAILURE => Self::DecryptionFailure,
            s::DAVE_DECRYPTOR_RESULT_CODE_MISSING_KEY_RATCHET => Self::MissingKeyRatchet,
            s::DAVE_DECRYPTOR_RESULT_CODE_INVALID_NONCE => Self::InvalidNonce,
            s::DAVE_DECRYPTOR_RESULT_CODE_MISSING_CRYPTOR => Self::MissingCryptor,
            other => Self::Unknown(other),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateFailed(what) => write!(f, "libdave failed to create {what}"),
            Self::InvalidArgument(what) => write!(f, "invalid argument: {what}"),
            Self::Encrypt(e) => write!(f, "DAVE encryption failed: {e:?}"),
            Self::Decrypt(e) => write!(f, "DAVE decryption failed: {e:?}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
