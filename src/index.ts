/**
 * Node.js bindings for Discord's DAVE end-to-end encryption, powered by the
 * official libdave through Rust and Node-API.
 *
 * @packageDocumentation
 */
export {
  Codec,
  type CommitOutcome,
  DaveSessionManager,
  Decryptor,
  type DecryptStats,
  Encryptor,
  type EncryptStats,
  KeyRatchet,
  type LogEntry,
  LogSeverity,
  MediaType,
  type MlsCommitResult,
  type MlsFailure,
  type MlsWelcomeResult,
  maxSupportedProtocolVersion,
  Session,
  setLogSink,
  type WelcomeOutcome,
} from '../binding.cjs';
