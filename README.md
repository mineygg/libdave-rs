# libdave-rs

Rust bindings to Discord's [DAVE](https://github.com/discord/libdave) end-to-end encryption
(MLS + per-frame media encryption) for voice/video.

| Crate | What it is |
|-------|-----------|
| `libdave-sys` | Raw FFI. `build.rs` compiles libdave + mlspp + nlohmann/json from pinned git submodules (no CMake) and links OpenSSL (vendored by default). |
| `libdave` | Safe wrappers (`Session`, `Encryptor`, `Decryptor`, `KeyRatchet`) plus `SessionManager`, the voice-gateway transition/epoch logic ported from cherrycord's `vocalize`. |
| `dave-cli` | Small binary (`version`, `selftest`, `keypackage`) built for every platform by CI. |

## Building

```sh
git clone --recurse-submodules <repo>   # or: git submodule update --init
cargo build --release
```

Needs a C++17 compiler, Perl and make (for vendored OpenSSL). Feature flags on `libdave`/`libdave-sys`:
`vendored-openssl` (default; disable to link a system OpenSSL >= 3) and `persistent-keys`.

## CI

* `ci.yml` - actionlint, rustfmt, clippy (`-D warnings`), rustdoc, tests on Linux/macOS/Windows, and the cross-build matrix.
* `build.yml` - reusable cross-compile matrix: Linux gnu/musl x64, arm64, armv7 via `cargo-zigbuild`; macOS arm64/x64; Windows x64/arm64. Each binary runs `dave-cli selftest` (natively or under qemu-user).
* `release.yml` - on `v*` tags, builds everything and publishes archives + `SHA256SUMS.txt`.

Rust builds are cached with `Swatinem/rust-cache` (per target).
