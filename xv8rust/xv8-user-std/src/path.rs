use alloc::string::ToString;
use alloc::vec::Vec;
use crate::ffi::{CString, OsStr};

pub struct Path { inner: [u8] }

impl Path {
    pub fn new<S: AsRef<[u8]> + ?Sized>(s: &S) -> &Path {
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
    pub fn file_stem(&self) -> Option<&Path> {
        let name = self.file_name()?;
        let s = name.to_str()?;
        let stem = s.rfind('.').map(|i| &s[..i]).unwrap_or(s);
        Some(Path::new(stem))
    }
    pub fn parent(&self) -> Option<&Path> {
        let s = self.to_str()?;
        s.rfind('/').map(|i| Path::new(&s[..i]))
    }
    pub fn is_absolute(&self) -> bool {
        self.inner.first() == Some(&b'/')
    }
    pub fn is_relative(&self) -> bool { !self.is_absolute() }
    pub fn is_dir(&self) -> bool { self.exists() && self.metadata().map(|m| m.is_dir()).unwrap_or(false) }
    pub fn is_file(&self) -> bool { self.exists() && self.metadata().map(|m| m.is_file()).unwrap_or(false) }
    pub fn as_os_str(&self) -> &OsStr {
        OsStr::from_str(core::str::from_utf8(&self.inner).unwrap_or(""))
    }
    pub fn ends_with(&self, other: &Path) -> bool {
        let s = match self.to_str() { Some(s) => s, None => return false };
        let o = match other.to_str() { Some(o) => o, None => return false };
        s.ends_with(o)
    }
    pub fn starts_with(&self, other: &Path) -> bool {
        let s = match self.to_str() { Some(s) => s, None => return false };
        let o = match other.to_str() { Some(o) => o, None => return false };
        if s == o { return true; }
        if o.is_empty() { return true; }
        s.starts_with(o) && (o.ends_with('/') || s.len() == o.len() || s.as_bytes().get(o.len()) == Some(&b'/'))
    }
    pub fn join<P: AsRef<Path>>(&self, other: P) -> PathBuf {
        let other = other.as_ref();
        if other.is_absolute() {
            return other.to_path_buf();
        }

        let mut inner = self.inner.as_ref().to_vec();
        if !inner.is_empty() && !inner.ends_with(&[b'/']) {
            inner.push(b'/');
        }
        inner.extend_from_slice(other.as_os_str().as_encoded_bytes());
        PathBuf { inner }
    }
    pub fn metadata(&self) -> super::io::Result<super::fs::Metadata> {
        let path_str = self.to_str().unwrap_or("");
        let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
        let fd = xv8_libc::open(c_path.as_ptr() as *const u8, xv8_libc::OpenFlag::READ_ONLY);
        if fd < 0 {
            Err(super::io::ErrorKind::NotFound.into())
        } else {
            let meta = super::fs::Metadata::from_fd(fd as usize);
            xv8_libc::close(fd as usize);
            meta
        }
    }
    pub fn to_path_buf(&self) -> PathBuf { PathBuf { inner: self.inner.as_ref().to_vec() } }
    pub fn to_string_lossy(&self) -> alloc::string::String { self.to_str().unwrap_or("?").to_string() }
    pub fn display(&self) -> DisplayPath<'_> { DisplayPath(self) }
    pub fn exists(&self) -> bool {
        let path_str = self.to_str().unwrap_or("");
        let Ok(c_path) = CString::new(path_str) else {
            return false;
        };
        let fd = xv8_libc::open(c_path.as_ptr() as *const u8, xv8_libc::OpenFlag::READ_ONLY);
        if fd >= 0 {
            xv8_libc::close(fd as usize);
            true
        } else {
            false
        }
    }
}

impl<'a> Default for &'a Path {
    fn default() -> &'a Path { Path::new("") }
}

impl core::convert::AsRef<Path> for Path {
    fn as_ref(&self) -> &Path { self }
}

impl core::convert::AsRef<Path> for str {
    fn as_ref(&self) -> &Path { Path::new(self) }
}

impl core::convert::AsRef<Path> for alloc::string::String {
    fn as_ref(&self) -> &Path { Path::new(self.as_str()) }
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
    pub fn push(&mut self, path: &Path) {
        if path.is_absolute() {
            self.inner = path.inner.as_ref().to_vec();
            return;
        }
        if !self.inner.is_empty() && self.inner.last() != Some(&b'/') {
            self.inner.push(b'/');
        }
        self.inner.extend_from_slice(&path.inner);
    }
    pub fn as_path(&self) -> &Path { Path::new(&self.inner) }
    pub fn pop(&mut self) -> bool {
        let s = match core::str::from_utf8(&self.inner) {
            Ok(s) => s,
            Err(_) => return false,
        };
        if s.is_empty() || s == "/" { return false; }
        let trimmed = s.trim_end_matches('/');
        if trimmed.is_empty() { // path was all slashes
            self.inner.truncate(1); // keep root
            return true;
        }
        match trimmed.rfind('/') {
            Some(pos) => {
                self.inner.truncate(pos + 1); // keep trailing /
                true
            }
            None => {
                self.inner.clear();
                true
            }
        }
    }
    pub fn set_file_name(&mut self, name: &str) {
        self.pop();
        if !name.is_empty() {
            if !self.inner.is_empty() && self.inner.last() != Some(&b'/') {
                self.inner.push(b'/');
            }
            self.inner.extend_from_slice(name.as_bytes());
        }
    }
    pub fn with_extension(&self, ext: &str) -> Self {
        let s = match self.to_str() {
            Some(s) => s,
            None => return self.clone(),
        };
        let dot = s.rfind('.');
        let base = match dot {
            Some(pos) if pos > s.rfind('/').map(|p| p + 1).unwrap_or(0) => &s[..pos],
            _ => s.trim_end_matches('/'),
        };
        if ext.is_empty() {
            PathBuf::from(base)
        } else {
            PathBuf::from(alloc::format!("{}.{}", base, ext))
        }
    }
    pub fn to_str(&self) -> Option<&str> { self.as_path().to_str() }
    pub fn to_string_lossy(&self) -> alloc::string::String { self.as_path().to_string_lossy() }
    pub fn as_os_str(&self) -> &OsStr { self.as_path().as_os_str() }
    pub fn exists(&self) -> bool { self.as_path().exists() }
    pub fn is_dir(&self) -> bool { self.as_path().is_dir() }
    pub fn is_file(&self) -> bool { self.as_path().is_file() }
    pub fn display(&self) -> DisplayPath<'_> { DisplayPath(self.as_path()) }
}

impl Default for PathBuf {
    fn default() -> Self { Self::new() }
}

impl Clone for PathBuf {
    fn clone(&self) -> Self { PathBuf { inner: self.inner.clone() } }
}

impl core::convert::AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path { self.as_path() }
}

impl core::ops::Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Path { self.as_path() }
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
