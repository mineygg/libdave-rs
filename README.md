# @mineygg/libdave-rs

Prebuilt Node.js bindings for Discord's official
[libdave](https://github.com/discord/libdave) (DAVE end-to-end encryption for
voice and video), written in Rust with [napi-rs](https://napi.rs).

```sh
npm install @mineygg/libdave-rs
```

npm installs only the binary for your machine, through `optionalDependencies`.
No compiler is needed.

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
| `crates/libdave`, `crates/libdave-sys` | Internal safe wrapper and FFI build of libdave          |

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

Set the `NPM_TOKEN` repository secret, then:

```sh
npm version patch   # also bumps the platform packages
git push --follow-tags
```
