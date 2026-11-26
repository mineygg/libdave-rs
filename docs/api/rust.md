# Rust crate overview

The repository exposes the Rust crate `libdave-rs` (the library crate is imported as `libdave_rs`) and a lower-level `libdave-rs-sys` crate.

## Install

```sh
cargo add libdave-rs
```

## Public exports

The safe crate's root exports, as observed in `crates/libdave/src/lib.rs`, include:

- `Session`
- `Encryptor`
- `Decryptor`
- `KeyRatchet`
- `SessionManager`
- `MediaType`
- `Codec`
- `max_supported_protocol_version`
- `LogSeverity`
- `LogSink`
- `set_log_sink`
- `CommitResult`
- `WelcomeResult`
- `CommitOutcome`
- `WelcomeOutcome`
- `FailureHandler`
- `EncryptorStats`
- `DecryptorStats`
- `Error`, `Result`, `EncryptError`, and `DecryptError`

`Session`, `Encryptor`, `Decryptor`, and `KeyRatchet` are described by the crate as RAII wrappers over the C API. `SessionManager` implements voice-gateway transition and epoch bookkeeping on top of those primitives.

## Design notes

- The safe crate statically builds libdave, mlspp, and OpenSSL from source according to its crate-level documentation; system copies of those libraries are not required.
- `MediaType` contains `Audio` and `Video`.
- `Codec` contains `Unknown`, `Opus`, `Vp8`, `Vp9`, `H264`, `H265`, and `Av1`.
- `max_supported_protocol_version()` returns the highest version supported by the linked libdave.
- `set_log_sink` installs or removes a process-wide logging callback. The callback receives severity, source file, line, and message.

## Rust API detail

This page deliberately avoids guessing method signatures that are not part of the root module declaration. For exact signatures, trait bounds, ownership, error variants, and examples, generate/read rustdoc from the same source revision:

```sh
cargo doc --no-deps --open
```

For the Node.js binding reference, see [Node.js API](nodejs.md).
