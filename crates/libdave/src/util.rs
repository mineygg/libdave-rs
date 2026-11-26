use std::ffi::CString;
use std::os::raw::{c_char, c_void};

use crate::error::{Error, Result};

pub(crate) fn cstring(s: &str, what: &'static str) -> Result<CString> {
    CString::new(s).map_err(|_| Error::InvalidArgument(what))
}

/// Copies a buffer allocated by libdave into a `Vec` and releases it with `daveFree`.
///
/// # Safety
/// `ptr` must be null or point to `len` bytes allocated by libdave.
pub(crate) unsafe fn take_bytes(ptr: *mut u8, len: usize) -> Option<Vec<u8>> {
    if ptr.is_null() {
        return None;
    }
    let v = if len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec()
    };
    unsafe { libdave_sys::daveFree(ptr.cast::<c_void>()) };
    Some(v)
}

/// Same as [`take_bytes`] for `u64` arrays (roster member ids).
///
/// # Safety
/// `ptr` must be null or point to `len` `u64`s allocated by libdave.
pub(crate) unsafe fn take_u64s(ptr: *mut u64, len: usize) -> Vec<u64> {
    if ptr.is_null() {
        return Vec::new();
    }
    let v = if len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec()
    };
    unsafe { libdave_sys::daveFree(ptr.cast::<c_void>()) };
    v
}

/// Owns a set of C strings plus the pointer array libdave expects.
pub(crate) struct CStringArray {
    _strings: Vec<CString>,
    ptrs: Vec<*const c_char>,
}

impl CStringArray {
    pub(crate) fn new<S: AsRef<str>>(items: &[S]) -> Result<Self> {
        let strings = items
            .iter()
            .map(|s| cstring(s.as_ref(), "user id contains a NUL byte"))
            .collect::<Result<Vec<_>>>()?;
        let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
        Ok(Self {
            _strings: strings,
            ptrs,
        })
    }

    pub(crate) fn as_ptr(&self) -> *const *const c_char {
        self.ptrs.as_ptr()
    }

    pub(crate) fn len(&self) -> usize {
        self.ptrs.len()
    }
}

pub(crate) unsafe fn cstr_lossy(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        unsafe { std::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .into_owned()
    }
}
