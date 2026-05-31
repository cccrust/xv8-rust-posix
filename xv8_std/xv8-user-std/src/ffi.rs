use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use crate::path::Path;

pub struct OsStr {
    inner: [u8],
}

impl OsStr {
    pub fn from_str(s: &str) -> &OsStr {
        unsafe { &*(s.as_bytes() as *const [u8] as *const OsStr) }
    }
    pub fn to_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.inner)
    }
    pub fn to_string_lossy(&self) -> alloc::string::String {
        self.to_str().unwrap_or("<invalid>").to_string()
    }
    pub fn as_encoded_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl core::fmt::Debug for OsStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.to_str().fmt(f)
    }
}

impl PartialEq for OsStr {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for OsStr {}

impl PartialOrd for OsStr {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OsStr {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl core::convert::AsRef<Path> for OsStr {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

pub struct CStr {
    inner: [u8],
}

impl CStr {
    pub unsafe fn from_ptr<'a>(ptr: *const i8) -> &'a CStr {
        let len = xv8_libc::strlen(ptr as *const u8);
        &*(core::slice::from_raw_parts(ptr as *const u8, len + 1) as *const [u8] as *const CStr)
    }
    pub fn to_bytes(&self) -> &[u8] {
        &self.inner[..self.inner.len() - 1]
    }
    pub fn to_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.to_bytes())
    }
    pub fn to_string_lossy(&self) -> alloc::string::String {
        self.to_str().unwrap_or("<invalid utf8>").to_string()
    }
}

pub struct CString {
    inner: Vec<u8>,
}

impl CString {
    pub fn new(s: &str) -> Result<Self, NulError> {
        if s.bytes().any(|b| b == 0) {
            return Err(NulError(()));
        }
        let mut v = Vec::with_capacity(s.len() + 1);
        v.extend_from_slice(s.as_bytes());
        v.push(0);
        Ok(CString { inner: v })
    }
    pub fn as_c_str(&self) -> &CStr {
        unsafe { &*(self.inner.as_slice() as *const [u8] as *const CStr) }
    }
    pub fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr() as *const i8
    }
}

impl Default for CString {
    fn default() -> Self { CString { inner: Vec::new() } }
}

#[derive(Debug, Clone)]
pub struct NulError(());

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nul byte found in string")
    }
}
