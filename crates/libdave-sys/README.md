# libdave-sys

Raw FFI bindings to Discord's official [libdave](https://github.com/discord/libdave),
built from bundled C++ sources (libdave, mlspp and nlohmann/json).

Most users want the safe [`libdave`](https://crates.io/crates/libdave) crate instead.

Building needs a C++17 compiler. The default `vendored-openssl` feature also needs
Perl and make; disable it to link a system OpenSSL 3.

Licensed under MIT. The bundled libraries keep their own licenses under `vendor/`.
