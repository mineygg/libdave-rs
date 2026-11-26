# Integration notes and lifecycle

## 1. Preserve snowflake precision

Discord IDs are unsigned 64-bit values. JavaScript `number` cannot represent every 64-bit integer exactly. Pass IDs as decimal strings; the manager's `groupId` additionally accepts `bigint`.

## 2. Treat gateway messages as protocol state

`DaveSessionManager` is not a generic encrypt/decrypt wrapper. It expects the caller to deliver voice-gateway protocol acknowledgements, transition events, epoch events, external-sender data, and MLS messages in the correct protocol order.

The documentation's snippets are intentionally schematic. Follow Discord's current DAVE voice protocol and gateway documentation for message transport and sequencing.

## 3. Distinguish no-frame from thrown errors

- Manager `encrypt`/`decrypt`: `Buffer | null` when the operation cannot produce a frame.
- Standalone `Encryptor.encrypt`/`Decryptor.decrypt`: return a `Buffer` on success and throw when libdave rejects the operation.
- Methods on destroyed wrappers throw.

Handle both `null` and exceptions where applicable.

## 4. Drain MLS failures

Call `takeFailures()` at suitable points in your event-processing loop or diagnostics path. Each call returns the current queued failures and empties the queue. The queue is capped at 64 entries.

## 5. Explicitly destroy native wrappers

Call `destroy()` on `DaveSessionManager`, `Session`, `Encryptor`, and `Decryptor` when they are no longer needed. This releases the native object immediately. Do not call methods after destruction.

`KeyRatchet` is an opaque handle in the Node.js API and does not expose `destroy()`.

## 6. Observe logs and counters

Use `setLogSink` to route native logs to your logger. Use `getEncryptStats`, `getDecryptStats`, or the standalone wrappers' `getStats` methods to observe media processing counters.

## 7. Platform support

The repository README lists these prebuilt targets:

| Platform       | Architectures     |
| -------------- | ----------------- |
| Linux glibc    | x64, arm64, armv7 |
| Linux musl     | x64, arm64, armv7 |
| macOS          | x64, arm64        |
| Windows (MSVC) | x64, arm64        |

The package declares Node.js `>=18`.
