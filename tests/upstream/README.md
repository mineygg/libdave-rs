# libdave upstream tests

These are the official tests from Discord's
[libdave](https://github.com/discord/libdave), run unmodified. No test code is
copied into this repository: it is read from the pinned submodule in
`crates/libdave-sys/vendor/libdave/cpp/test`, so the tests always match the
libdave version that `libdave-rs` ships.

The tests link against the static libraries produced by
`crates/libdave-sys/build.rs` (libdave, mlspp and OpenSSL), not against a
separate upstream CMake build.

## Run

Needs Linux or macOS, Rust, CMake 3.20+, a C++17 compiler, Perl, make and
GoogleTest with GoogleMock.

```sh
# Debian / Ubuntu
sudo apt-get install cmake libgtest-dev libgmock-dev
# macOS
brew install cmake googletest

git submodule update --init
npm run test:upstream
```

The command downloads nothing. It stops if the libdave submodule is not at the
pinned commit or has local changes.

## Scope

- `libdave_test`: the C++ tests, including the fuzz corpus regression tests.
- `capi_test`: the C API tests.
- Only the default build is tested, without the `persistent-keys` feature.
- Windows is not supported.

When updating the submodule, compare `CMakeLists.txt` here with upstream's
`cpp/test/CMakeLists.txt` and `cpp/test/capi/CMakeLists.txt`, and with the
include paths in `build.rs`.

## Badge

The test run writes its counts to `target/upstream-tests/results.json`. On `main`,
CI publishes them with the build count to the `badges` branch, which the README
badges read. The separate `tests` badge counts this repository's own Rust and
Node tests.
