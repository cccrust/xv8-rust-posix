use alloc::string::{String, ToString};

use crate::ffi::CString;

pub struct EnvArgs {
    inner: xv8_libc::args::Args,
    current: usize,
}

impl EnvArgs {
    fn new() -> Self {
        EnvArgs { inner: unsafe { xv8_libc::args::Args::from_stack() }, current: 0 }
    }
}

impl Iterator for EnvArgs {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.current >= self.inner.argc { return None; }
        let arg = self.inner.get_str(self.current)?;
        self.current += 1;
        Some(arg.to_string())
    }
}

pub fn args() -> EnvArgs { EnvArgs::new() }

#[derive(Debug, PartialEq, Eq)]
pub enum VarError {
    NotPresent,
    NotUnicode(alloc::string::String),
}

pub struct EmptyEnvVars;
impl Iterator for EmptyEnvVars {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> { None }
}

pub fn vars() -> EmptyEnvVars { EmptyEnvVars }

pub fn current_exe() -> super::io::Result<super::path::PathBuf> {
    Ok(super::path::PathBuf::from(b"/bin/sh"))
}

pub fn current_dir() -> super::io::Result<super::path::PathBuf> {
    match var("PWD") {
        Ok(path) if !path.is_empty() => Ok(super::path::PathBuf::from(path.as_bytes())),
        _ => Ok(super::path::PathBuf::from(b"/")),
    }
}

pub fn set_current_dir(path: &super::path::Path) -> super::io::Result<()> {
    let path_str = path.to_str().unwrap_or("");
    let c_path = CString::new(path_str).map_err(|_| super::io::ErrorKind::InvalidInput)?;
    let n = xv8_libc::chdir(c_path.as_ptr() as *const u8);
    if n < 0 {
        Err(super::io::ErrorKind::Other.into())
    } else {
        unsafe { set_var("PWD", path_str); }
        Ok(())
    }
}

pub fn var(key: &str) -> Result<String, VarError> {
    let c_key = CString::new(key).map_err(|_| VarError::NotPresent)?;
    let mut buffer = [0u8; 256];
    let n = xv8_libc::getenv(c_key.as_ptr() as *const u8, buffer.as_mut_ptr(), buffer.len());
    if n < 0 {
        return Err(VarError::NotPresent);
    }
    let value = &buffer[..n as usize];
    String::from_utf8(value.to_vec()).map_err(|_| VarError::NotUnicode(String::from_utf8_lossy(value).to_string()))
}

pub unsafe fn set_var(key: &str, value: &str) {
    if let (Ok(c_key), Ok(c_value)) = (CString::new(key), CString::new(value)) {
        let _ = xv8_libc::setenv(c_key.as_ptr() as *const u8, c_value.as_ptr() as *const u8, 1);
    }
}

pub unsafe fn remove_var(key: &str) {
    if let Ok(c_key) = CString::new(key) {
        let _ = xv8_libc::unsetenv(c_key.as_ptr() as *const u8);
    }
}
