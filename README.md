# @mineygg/libdave-rs

Prebuilt Node.js bindings for Discord's official
[libdave](https://github.com/discord/libdave) (DAVE end-to-end encryption for
voice and video), written in Rust with [napi-rs](https://napi.rs).

```sh
npm install @mineygg/libdave-rs
```

npm installs only the binary for your machine, through `optionalDependencies`.
No compiler is needed.

## Documentation

**[Read the full documentation](https://github.com/mineygg/libdave-rs/tree/main/docs)**

The documentation includes the getting-started guide, Node.js API reference,
class and type references, Rust crate overview, and integration notes.

## Usage

```ts
import {
  DaveSessionManager,
  MediaType,
  maxSupportedProtocolVersion,
} from '@mineygg/libdave-rs';

const dave = new DaveSessionManager(selfUserId, channelId);
dave.addUser(otherUserId);
dave.onSelectProtocolAck(maxSupportedProtocolVersion());
const keyPackage = dave.getMarshalledKeyPackage(); // send to the voice gateway
const encrypted = dave.encrypt(MediaType.Audio, ssrc, opusFrame); // Buffer | null
const plain = dave.decrypt(MediaType.Audio, otherUserId, received); // Buffer | null
```

CommonJS `require` works too. The lower-level `Session`, `Encryptor`,
`Decryptor` and `KeyRatchet` classes are also exported. MLS failures are read
with `takeFailures()`.

## Rust

The same library is on crates.io:

```sh
cargo add libdave-rs
```

## Platforms

Each is published as `@mineygg/libdave-rs-<platform>`.

| OS            | x64              | arm64              | armv7                  |
| ------------- | ---------------- | ------------------ | ---------------------- |
| Linux (glibc) | `linux-x64-gnu`  | `linux-arm64-gnu`  | `linux-arm-gnueabihf`  |
| Linux (musl)  | `linux-x64-musl` | `linux-arm64-musl` | `linux-arm-musleabihf` |
| macOS         | `darwin-x64`     | `darwin-arm64`     |                        |
| Windows       | `win32-x64-msvc` | `win32-arm64-msvc` |                        |

## Layout

| Path                                   | What it is                                              |
| -------------------------------------- | ------------------------------------------------------- |
| `src/`                                 | TypeScript entry point, bundled with tsup (ESM and CJS) |
| `npm/`                                 | One small package per platform                          |
| `crates/libdave-node`                  | Node-API layer                                          |
| `crates/libdave`, `crates/libdave-sys` | The `libdave-rs` and `libdave-rs-sys` Rust crates       |

## Development

```sh
git submodule update --init
npm ci
npm run build      # native addon (needs Rust, C++17, Perl, make) + TypeScript
npm test           # vitest
npm run lint       # biome
npm run typecheck
```

To work on TypeScript only, download the `bindings-<platform>` and `js-binding`
artifacts from a CI run into the repository root and skip the native build.
Git hooks are managed by lefthook.

## Releasing

Set the `NPM_TOKEN` and `CARGO_REGISTRY_TOKEN` repository secrets, then run the
commands below. This publishes the npm packages and the `libdave-rs-sys` and
`libdave-rs` crates.

```sh
npm version patch   # also bumps the platform packages and crates
git push --follow-tags
```

## License

This project is MIT licensed. The published binaries and `libdave-rs-sys` include
third-party code under its own terms, listed in `THIRD_PARTY_LICENSES`. After
changing dependencies or submodules, run `npm run licenses`.
