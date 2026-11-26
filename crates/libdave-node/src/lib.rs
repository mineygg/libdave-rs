//! Node-API layer over the [`libdave`] crate. Published to npm as `@mineygg/libdave-rs`.
// napi's registration code is compiled out under `cfg(test)`, which `clippy --all-targets`
// would otherwise report as dead code.
#![cfg_attr(test, allow(dead_code))]

mod crypto;
mod log;
mod manager;
mod session;
mod types;

use napi::bindgen_prelude::*;

pub(crate) fn to_napi(error: libdave::Error) -> Error {
    Error::from_reason(error.to_string())
}

pub(crate) fn destroyed(what: &str) -> Error {
    Error::from_reason(format!("{what} has been destroyed"))
}

/// Accepts a snowflake as a decimal string or a bigint.
pub(crate) fn snowflake(value: Either<String, BigInt>, what: &str) -> Result<u64> {
    let invalid = || {
        Error::new(
            Status::InvalidArg,
            format!("{what} must be an unsigned 64-bit integer"),
        )
    };
    match value {
        Either::A(text) => text.parse().map_err(|_| invalid()),
        Either::B(big) => match big.get_u64() {
            (false, number, true) => Ok(number),
            _ => Err(invalid()),
        },
    }
}

/// Highest DAVE protocol version supported by the bundled libdave.
#[napi_derive::napi]
pub fn max_supported_protocol_version() -> u16 {
    libdave::max_supported_protocol_version()
}
