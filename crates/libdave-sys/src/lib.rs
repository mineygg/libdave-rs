//! Raw FFI bindings to Discord's [libdave](https://github.com/discord/libdave) C API
//! (`dave/dave.h`).
//!
//! libdave, mlspp and (by default) OpenSSL are compiled from source and linked statically.
//! Use the safe `libdave` crate unless you really need the raw interface.
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

// Make sure openssl-sys (and therefore libssl/libcrypto) ends up on the link line.
extern crate openssl_sys;

use std::os::raw::{c_char, c_int, c_void};

macro_rules! opaque_handle {
    ($($name:ident => $ptr:ident),* $(,)?) => {$(
        #[repr(C)]
        pub struct $name { _private: [u8; 0] }
        pub type $ptr = *mut $name;
    )*};
}

opaque_handle! {
    DAVESessionHandle_s => DAVESessionHandle,
    DAVECommitResultHandle_s => DAVECommitResultHandle,
    DAVEWelcomeResultHandle_s => DAVEWelcomeResultHandle,
    DAVEKeyRatchetHandle_s => DAVEKeyRatchetHandle,
    DAVEEncryptorHandle_s => DAVEEncryptorHandle,
    DAVEDecryptorHandle_s => DAVEDecryptorHandle,
}

pub type DAVECodec = u32;
pub const DAVE_CODEC_UNKNOWN: DAVECodec = 0;
pub const DAVE_CODEC_OPUS: DAVECodec = 1;
pub const DAVE_CODEC_VP8: DAVECodec = 2;
pub const DAVE_CODEC_VP9: DAVECodec = 3;
pub const DAVE_CODEC_H264: DAVECodec = 4;
pub const DAVE_CODEC_H265: DAVECodec = 5;
pub const DAVE_CODEC_AV1: DAVECodec = 6;

pub type DAVEMediaType = u32;
pub const DAVE_MEDIA_TYPE_AUDIO: DAVEMediaType = 0;
pub const DAVE_MEDIA_TYPE_VIDEO: DAVEMediaType = 1;

pub type DAVEEncryptorResultCode = u32;
pub const DAVE_ENCRYPTOR_RESULT_CODE_SUCCESS: DAVEEncryptorResultCode = 0;
pub const DAVE_ENCRYPTOR_RESULT_CODE_ENCRYPTION_FAILURE: DAVEEncryptorResultCode = 1;
pub const DAVE_ENCRYPTOR_RESULT_CODE_MISSING_KEY_RATCHET: DAVEEncryptorResultCode = 2;
pub const DAVE_ENCRYPTOR_RESULT_CODE_MISSING_CRYPTOR: DAVEEncryptorResultCode = 3;
pub const DAVE_ENCRYPTOR_RESULT_CODE_TOO_MANY_ATTEMPTS: DAVEEncryptorResultCode = 4;

pub type DAVEDecryptorResultCode = u32;
pub const DAVE_DECRYPTOR_RESULT_CODE_SUCCESS: DAVEDecryptorResultCode = 0;
pub const DAVE_DECRYPTOR_RESULT_CODE_DECRYPTION_FAILURE: DAVEDecryptorResultCode = 1;
pub const DAVE_DECRYPTOR_RESULT_CODE_MISSING_KEY_RATCHET: DAVEDecryptorResultCode = 2;
pub const DAVE_DECRYPTOR_RESULT_CODE_INVALID_NONCE: DAVEDecryptorResultCode = 3;
pub const DAVE_DECRYPTOR_RESULT_CODE_MISSING_CRYPTOR: DAVEDecryptorResultCode = 4;

pub type DAVELoggingSeverity = u32;
pub const DAVE_LOGGING_SEVERITY_VERBOSE: DAVELoggingSeverity = 0;
pub const DAVE_LOGGING_SEVERITY_INFO: DAVELoggingSeverity = 1;
pub const DAVE_LOGGING_SEVERITY_WARNING: DAVELoggingSeverity = 2;
pub const DAVE_LOGGING_SEVERITY_ERROR: DAVELoggingSeverity = 3;
pub const DAVE_LOGGING_SEVERITY_NONE: DAVELoggingSeverity = 4;

pub type DAVEMLSFailureCallback =
    unsafe extern "C" fn(source: *const c_char, reason: *const c_char, user_data: *mut c_void);
pub type DAVEPairwiseFingerprintCallback =
    unsafe extern "C" fn(fingerprint: *const u8, length: usize, user_data: *mut c_void);
pub type DAVEEncryptorProtocolVersionChangedCallback = unsafe extern "C" fn(user_data: *mut c_void);
pub type DAVELogSinkCallback = unsafe extern "C" fn(
    severity: DAVELoggingSeverity,
    file: *const c_char,
    line: c_int,
    message: *const c_char,
);

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DAVEEncryptorStats {
    pub passthroughCount: u64,
    pub encryptSuccessCount: u64,
    pub encryptFailureCount: u64,
    pub encryptDuration: u64,
    pub encryptAttempts: u64,
    pub encryptMaxAttempts: u64,
    pub encryptMissingKeyCount: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DAVEDecryptorStats {
    pub passthroughCount: u64,
    pub decryptSuccessCount: u64,
    pub decryptFailureCount: u64,
    pub decryptDuration: u64,
    pub decryptAttempts: u64,
    pub decryptMissingKeyCount: u64,
    pub decryptInvalidNonceCount: u64,
}

extern "C" {
    // Version / memory
    pub fn daveMaxSupportedProtocolVersion() -> u16;
    pub fn daveFree(ptr: *mut c_void);

    // Session
    pub fn daveSessionCreate(
        context: *mut c_void,
        authSessionId: *const c_char,
        callback: Option<DAVEMLSFailureCallback>,
        userData: *mut c_void,
    ) -> DAVESessionHandle;
    pub fn daveSessionDestroy(session: DAVESessionHandle);
    pub fn daveSessionInit(
        session: DAVESessionHandle,
        version: u16,
        groupId: u64,
        selfUserId: *const c_char,
    );
    pub fn daveSessionReset(session: DAVESessionHandle);
    pub fn daveSessionSetProtocolVersion(session: DAVESessionHandle, version: u16);
    pub fn daveSessionGetProtocolVersion(session: DAVESessionHandle) -> u16;
    pub fn daveSessionGetLastEpochAuthenticator(
        session: DAVESessionHandle,
        authenticator: *mut *mut u8,
        length: *mut usize,
    );
    pub fn daveSessionSetExternalSender(
        session: DAVESessionHandle,
        externalSender: *const u8,
        length: usize,
    );
    pub fn daveSessionAddRecognizedUser(session: DAVESessionHandle, userId: *const c_char);
    pub fn daveSessionRemoveRecognizedUser(session: DAVESessionHandle, userId: *const c_char);
    pub fn daveSessionProcessProposals(
        session: DAVESessionHandle,
        proposals: *const u8,
        length: usize,
        commitWelcomeBytes: *mut *mut u8,
        commitWelcomeBytesLength: *mut usize,
    );
    pub fn daveSessionProcessCommit(
        session: DAVESessionHandle,
        commit: *const u8,
        length: usize,
    ) -> DAVECommitResultHandle;
    pub fn daveSessionProcessWelcome(
        session: DAVESessionHandle,
        welcome: *const u8,
        length: usize,
    ) -> DAVEWelcomeResultHandle;
    pub fn daveSessionGetMarshalledKeyPackage(
        session: DAVESessionHandle,
        keyPackage: *mut *mut u8,
        length: *mut usize,
    );
    pub fn daveSessionGetKeyRatchet(
        session: DAVESessionHandle,
        userId: *const c_char,
    ) -> DAVEKeyRatchetHandle;
    pub fn daveSessionGetPairwiseFingerprint(
        session: DAVESessionHandle,
        version: u16,
        userId: *const c_char,
        callback: DAVEPairwiseFingerprintCallback,
        userData: *mut c_void,
    );

    // Key ratchet
    pub fn daveKeyRatchetDestroy(keyRatchet: DAVEKeyRatchetHandle);

    // Commit result
    pub fn daveCommitResultIsFailed(handle: DAVECommitResultHandle) -> bool;
    pub fn daveCommitResultIsIgnored(handle: DAVECommitResultHandle) -> bool;
    pub fn daveCommitResultGetRosterMemberIds(
        handle: DAVECommitResultHandle,
        rosterIds: *mut *mut u64,
        rosterIdsLength: *mut usize,
    );
    pub fn daveCommitResultGetRosterMemberSignature(
        handle: DAVECommitResultHandle,
        rosterId: u64,
        signature: *mut *mut u8,
        signatureLength: *mut usize,
    );
    pub fn daveCommitResultDestroy(handle: DAVECommitResultHandle);

    // Welcome result
    pub fn daveWelcomeResultGetRosterMemberIds(
        handle: DAVEWelcomeResultHandle,
        rosterIds: *mut *mut u64,
        rosterIdsLength: *mut usize,
    );
    pub fn daveWelcomeResultGetRosterMemberSignature(
        handle: DAVEWelcomeResultHandle,
        rosterId: u64,
        signature: *mut *mut u8,
        signatureLength: *mut usize,
    );
    pub fn daveWelcomeResultDestroy(handle: DAVEWelcomeResultHandle);

    // Encryptor
    pub fn daveEncryptorCreate() -> DAVEEncryptorHandle;
    pub fn daveEncryptorDestroy(encryptor: DAVEEncryptorHandle);
    pub fn daveEncryptorSetKeyRatchet(
        encryptor: DAVEEncryptorHandle,
        keyRatchet: DAVEKeyRatchetHandle,
    );
    pub fn daveEncryptorSetPassthroughMode(encryptor: DAVEEncryptorHandle, passthroughMode: bool);
    pub fn daveEncryptorAssignSsrcToCodec(
        encryptor: DAVEEncryptorHandle,
        ssrc: u32,
        codecType: DAVECodec,
    );
    pub fn daveEncryptorGetProtocolVersion(encryptor: DAVEEncryptorHandle) -> u16;
    pub fn daveEncryptorGetMaxCiphertextByteSize(
        encryptor: DAVEEncryptorHandle,
        mediaType: DAVEMediaType,
        frameSize: usize,
    ) -> usize;
    pub fn daveEncryptorHasKeyRatchet(encryptor: DAVEEncryptorHandle) -> bool;
    pub fn daveEncryptorIsPassthroughMode(encryptor: DAVEEncryptorHandle) -> bool;
    pub fn daveEncryptorEncrypt(
        encryptor: DAVEEncryptorHandle,
        mediaType: DAVEMediaType,
        ssrc: u32,
        frame: *const u8,
        frameLength: usize,
        encryptedFrame: *mut u8,
        encryptedFrameCapacity: usize,
        bytesWritten: *mut usize,
    ) -> DAVEEncryptorResultCode;
    pub fn daveEncryptorSetProtocolVersionChangedCallback(
        encryptor: DAVEEncryptorHandle,
        callback: DAVEEncryptorProtocolVersionChangedCallback,
        userData: *mut c_void,
    );
    pub fn daveEncryptorGetStats(
        encryptor: DAVEEncryptorHandle,
        mediaType: DAVEMediaType,
        stats: *mut DAVEEncryptorStats,
    );

    // Decryptor
    pub fn daveDecryptorCreate() -> DAVEDecryptorHandle;
    pub fn daveDecryptorDestroy(decryptor: DAVEDecryptorHandle);
    pub fn daveDecryptorTransitionToKeyRatchet(
        decryptor: DAVEDecryptorHandle,
        keyRatchet: DAVEKeyRatchetHandle,
    );
    pub fn daveDecryptorTransitionToPassthroughMode(
        decryptor: DAVEDecryptorHandle,
        passthroughMode: bool,
    );
    pub fn daveDecryptorDecrypt(
        decryptor: DAVEDecryptorHandle,
        mediaType: DAVEMediaType,
        encryptedFrame: *const u8,
        encryptedFrameLength: usize,
        frame: *mut u8,
        frameCapacity: usize,
        bytesWritten: *mut usize,
    ) -> DAVEDecryptorResultCode;
    pub fn daveDecryptorGetMaxPlaintextByteSize(
        decryptor: DAVEDecryptorHandle,
        mediaType: DAVEMediaType,
        encryptedFrameSize: usize,
    ) -> usize;
    pub fn daveDecryptorGetStats(
        decryptor: DAVEDecryptorHandle,
        mediaType: DAVEMediaType,
        stats: *mut DAVEDecryptorStats,
    );

    // Logging
    pub fn daveSetLogSinkCallback(callback: Option<DAVELogSinkCallback>);
}
