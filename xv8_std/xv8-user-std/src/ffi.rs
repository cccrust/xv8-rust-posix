use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

pub struct CStr {
    inner: [u8],
}

impl CStr {
    pub unsafe fn from_ptr<'a>(ptr: *const u8) -> &'a CStr {
        let len = xv8_libc::strlen(ptr);
        &*(core::slice::from_raw_parts(ptr, len + 1) as *const [u8] as *const CStr)
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
}

#[derive(Debug, Clone)]
pub struct NulError(());

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nul byte found in string")
    }
}
