//! Builds libdave (Discord's DAVE implementation) together with its C++ dependencies
//! (mlspp + nlohmann/json) straight from the vendored git submodules, bypassing CMake.
//!
//! OpenSSL is provided by `openssl-sys` (vendored by default), which exports its include
//! directory to us through `DEP_OPENSSL_INCLUDE`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vendor = manifest.join("vendor");

    let libdave = vendor.join("libdave").join("cpp");
    let mlspp = vendor.join("mlspp");
    let nlohmann = vendor.join("nlohmann_json");

    if !libdave.join("src").join("bindings_capi.cpp").exists()
        || !mlspp.join("src").exists()
        || !nlohmann.join("include").exists()
    {
        panic!(
            "libdave-sys: vendored sources are missing. Run \
             `git submodule update --init` in the repository root."
        );
    }

    for dir in [
        libdave.join("src"),
        libdave.join("includes"),
        mlspp.join("src"),
        mlspp.join("include"),
        mlspp.join("lib"),
    ] {
        println!("cargo:rerun-if-changed={}", dir.display());
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let is_msvc = target_env == "msvc";
    let persistent_keys = env::var_os("CARGO_FEATURE_PERSISTENT_KEYS").is_some();

    // ---- OpenSSL (from openssl-sys) -------------------------------------------------------
    let openssl_include = env::var_os("DEP_OPENSSL_INCLUDE")
        .map(PathBuf::from)
        .expect("DEP_OPENSSL_INCLUDE not set (openssl-sys must be a direct dependency)");

    if let Ok(v) = env::var("DEP_OPENSSL_VERSION_NUMBER") {
        let n = u64::from_str_radix(v.trim_start_matches("0x"), 16).unwrap_or(0x3000_0000);
        assert!(
            n >= 0x3000_0000,
            "libdave-sys requires OpenSSL >= 3.0 (found {v})"
        );
    }

    // mlspp expects a generated <namespace.h>; libdave refers to the namespace as `mlspp`.
    let gen_include = out_dir.join("include");
    fs::create_dir_all(&gen_include).unwrap();
    fs::write(
        gen_include.join("namespace.h"),
        "#pragma once\n\n#define MLS_NAMESPACE mlspp\n",
    )
    .unwrap();

    // ---- mlspp (+ bytes / tls_syntax / hpke) ------------------------------------------------
    let mut mlspp_sources = cpp_files(&mlspp.join("src"), false);
    let mut mlspp_includes = vec![
        gen_include.clone(),
        mlspp.join("include"),
        mlspp.join("src"),
        mlspp.join("third_party"),
        nlohmann.join("include"),
        openssl_include.clone(),
    ];
    for lib in ["bytes", "tls_syntax", "hpke"] {
        let lib_dir = mlspp.join("lib").join(lib);
        mlspp_sources.extend(cpp_files(&lib_dir.join("src"), false));
        mlspp_includes.push(lib_dir.join("include"));
        mlspp_includes.push(lib_dir.join("src"));
    }
    mlspp_sources.sort();

    let mut mlspp_build = base_build(is_msvc);
    for inc in &mlspp_includes {
        mlspp_build.include(inc);
    }
    mlspp_build.define("WITH_OPENSSL3", None);
    mlspp_build.files(&mlspp_sources);
    mlspp_build.compile("dave_mlspp");

    // ---- libdave ----------------------------------------------------------------------------
    let mut dave_sources: Vec<PathBuf> = cpp_files(&libdave.join("src"), true)
        .into_iter()
        .filter(|p| {
            let name = file_name(p);
            !name.starts_with("persisted_key")
                && !name.ends_with("_win.cpp")
                && !name.ends_with("_apple.cpp")
                && !name.ends_with("_wasm.cpp")
                && !name.contains("boringssl")
                && !name.contains("test")
        })
        .collect();

    // Mirror upstream CMake: pick exactly one persisted-key implementation set.
    let src = libdave.join("src").join("mls");
    if persistent_keys {
        dave_sources.push(src.join("persisted_key_pair.cpp"));
        if target_os == "macos" || target_os == "ios" {
            dave_sources.push(src.join("detail").join("persisted_key_pair_apple.cpp"));
        } else {
            dave_sources.push(src.join("detail").join("persisted_key_pair_generic.cpp"));
            if target_os == "windows" {
                dave_sources.push(src.join("detail").join("persisted_key_pair_win.cpp"));
            } else {
                dave_sources.push(src.join("detail").join("persisted_key_pair_null.cpp"));
            }
        }
    } else {
        dave_sources.push(src.join("persisted_key_pair_null.cpp"));
    }
    dave_sources.sort();

    let mut dave_build = base_build(is_msvc);
    // mlspp's public headers must win over libdave's own `mls/` directory.
    for inc in [
        libdave.join("includes"),
        gen_include.clone(),
        mlspp.join("include"),
        mlspp.join("src"),
        mlspp.join("third_party"),
        nlohmann.join("include"),
        libdave.join("src"),
        libdave.clone(),
        openssl_include.clone(),
    ] {
        dave_build.include(inc);
    }
    for lib in ["bytes", "tls_syntax", "hpke"] {
        dave_build.include(mlspp.join("lib").join(lib).join("include"));
    }
    dave_build.define("WITH_OPENSSL3", None);
    dave_build.files(&dave_sources);
    dave_build.compile("dave_core");

    // ---- platform libraries -----------------------------------------------------------------
    if persistent_keys {
        match target_os.as_str() {
            "macos" | "ios" => {
                println!("cargo:rustc-link-lib=framework=CoreFoundation");
                println!("cargo:rustc-link-lib=framework=Security");
            }
            "windows" => {
                println!("cargo:rustc-link-lib=ncrypt");
                println!("cargo:rustc-link-lib=crypt32");
            }
            _ => {}
        }
    }
}

fn base_build(is_msvc: bool) -> cc::Build {
    let mut b = cc::Build::new();
    b.cpp(true)
        .std("c++17")
        .opt_level(2)
        .debug(false)
        .warnings(false)
        .extra_warnings(false);
    b.define("NOMINMAX", None);
    if is_msvc {
        b.flag("/EHsc");
        b.define("WINDOWS", None);
        b.define("_CRT_SECURE_NO_WARNINGS", None);
    }
    b
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

/// Collects `*.cpp` / `*.cc` files below `dir`.
fn cpp_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if recursive {
                    stack.push(path);
                }
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("cpp" | "cc")
            ) {
                out.push(path);
            }
        }
    }
    out
}
