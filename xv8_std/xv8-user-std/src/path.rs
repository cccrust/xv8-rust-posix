use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct Path { inner: [u8] }

impl Path {
    pub fn new<S: AsRef<[u8]>>(s: S) -> &Path {
        Path::from_bytes(s.as_ref())
    }
    pub fn from_bytes(s: &[u8]) -> &Path {
        unsafe { &*(s as *const [u8] as *const Path) }
    }
    pub fn to_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.inner).ok()
    }
    pub fn file_name(&self) -> Option<&Path> {
        let s = self.to_str()?;
        s.rfind('/').map(|i| Path::new(&s[i+1..]))
    }
    pub fn parent(&self) -> Option<&Path> {
        let s = self.to_str()?;
        s.rfind('/').map(|i| Path::new(&s[..i]));
        None
    }
    pub fn is_absolute(&self) -> bool { false }
    pub fn is_relative(&self) -> bool { true }
    pub fn ends_with(&self, _other: &Path) -> bool { false }
    pub fn starts_with(&self, _other: &Path) -> bool { false }
    pub fn join(&self, _other: &Path) -> PathBuf { PathBuf { inner: self.inner.as_ref().to_vec() } }
    pub fn to_path_buf(&self) -> PathBuf { PathBuf { inner: self.inner.as_ref().to_vec() } }
    pub fn to_string_lossy(&self) -> alloc::string::String { self.to_str().unwrap_or("?").to_string() }
    pub fn display(&self) -> DisplayPath { DisplayPath(self) }
}

impl core::convert::AsRef<Path> for Path {
    fn as_ref(&self) -> &Path { self }
}

impl<'a> core::convert::TryFrom<&'a Path> for &'a str {
    type Error = ();
    fn try_from(p: &'a Path) -> Result<&'a str, Self::Error> {
        p.to_str().ok_or(())
    }
}

pub struct PathBuf { inner: Vec<u8> }

impl PathBuf {
    pub fn new() -> Self { PathBuf { inner: Vec::new() } }
    pub fn from<S: AsRef<[u8]>>(s: S) -> Self { PathBuf { inner: s.as_ref().to_vec() } }
    pub fn push(&mut self, _path: &Path) {}
    pub fn as_path(&self) -> &Path { Path::new(&self.inner) }
    pub fn pop(&mut self) -> bool { false }
    pub fn set_file_name(&mut self, _name: &str) {}
    pub fn with_extension(&self, _ext: &str) -> Self { self.clone() }
}

impl Default for PathBuf {
    fn default() -> Self { Self::new() }
}

impl Clone for PathBuf {
    fn clone(&self) -> Self { PathBuf { inner: self.inner.clone() } }
}

impl alloc::fmt::Display for Path {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("?"))
    }
}

impl alloc::fmt::Display for PathBuf {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{}", self.to_str().unwrap_or("?"))
    }
}

pub struct DisplayPath<'a>(&'a Path);
impl<'a> alloc::fmt::Display for DisplayPath<'a> {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{}", self.0.to_str().unwrap_or("?"))
    }
}