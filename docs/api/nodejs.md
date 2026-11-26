# Node.js API reference

## Package exports

The package entry point exports:

```ts
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
};
```

## Conventions

- Native byte slices are exposed as Node.js `Buffer` inputs and outputs.
- Methods that return `Buffer | null` use `null` when the operation produced no frame/message or could not be completed as described by that method.
- napi-rs errors are surfaced as JavaScript exceptions. Calls on explicitly destroyed native objects throw an error indicating that the object has been destroyed.
- IDs are generally passed as decimal strings. The manager's `groupId` accepts a decimal string or `bigint`.
- `u16`/`u32` values map to JavaScript numbers. Do not pass values outside the corresponding unsigned integer range.

## Top-level function

### `maxSupportedProtocolVersion(): number`

Returns the highest DAVE protocol version supported by the linked/bundled libdave.

## API pages

- [DaveSessionManager](dave-session-manager.md)
- [Session](session.md)
- [Media cryptography](crypto.md)
- [Enums, result objects, stats, failures, and logs](types-and-logging.md)
