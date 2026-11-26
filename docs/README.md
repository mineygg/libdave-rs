# libdave-rs documentation

Documentation for **`@mineygg/libdave-rs`**, the prebuilt Node.js bindings for Discord's DAVE end-to-end encryption implementation, plus an overview of the Rust crate.

## Contents

- [Getting started](guides/getting-started.md)
- [Node.js API reference](api/nodejs.md)
- [DaveSessionManager](api/dave-session-manager.md)
- [Session](api/session.md)
- [Encryptor, Decryptor, and KeyRatchet](api/crypto.md)
- [Types, statistics, failures, and logging](api/types-and-logging.md)
- [Rust crate overview](api/rust.md)
- [Integration notes and lifecycle](guides/integration.md)

## Scope and accuracy

The Node.js reference follows the repository's current `src/index.ts` exports and `crates/libdave-node/src/*.rs` napi-rs bindings. It documents the public JavaScript surface visible in those files.

The Rust section describes the public crate exports visible in `crates/libdave/src/lib.rs`. It is intentionally an overview rather than a fabricated signature-by-signature Rust reference. For exact Rust signatures and rustdoc, consult the crate source and generated rustdoc for the checked-out revision.

**Version context:** the repository metadata inspected for this documentation reports npm version `0.1.2`. APIs can change between revisions; regenerate or verify this reference when publishing a new release.

## Links

- Repository: https://github.com/mineygg/libdave-rs
- Upstream implementation: https://github.com/discord/libdave
- napi-rs: https://napi.rs/
