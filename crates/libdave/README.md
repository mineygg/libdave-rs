# libdave

Safe Rust bindings to Discord's official [libdave](https://github.com/discord/libdave),
the DAVE end-to-end encryption library for voice and video.

```sh
cargo add libdave
```

```rust
let version = libdave::max_supported_protocol_version();
let mut session = libdave::Session::new(None, None)?;
session.init(version, channel_id, &self_user_id)?;
let key_package = session.marshalled_key_package();
```

Building needs a C++17 compiler, plus Perl and make for the bundled OpenSSL.
Node.js users can install `@mineygg/libdave-rs` from npm.
