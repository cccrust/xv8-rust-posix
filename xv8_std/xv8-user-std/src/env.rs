use alloc::string::{String, ToString};
use user::syscall::Args;

pub struct EnvArgs {
    inner: Args,
    current: usize,
}

impl EnvArgs {
    fn new() -> Self {
        EnvArgs { inner: unsafe { Args::from_stack() }, current: 1 }
    }
}

impl Iterator for EnvArgs {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.current >= self.inner.len() { return None; }
        let arg = self.inner.get_str(self.current);
        self.current += 1;
        arg.map(|s| s.to_string())
    }
}

pub fn args() -> EnvArgs { EnvArgs::new() }

pub struct EmptyEnvVars;
impl Iterator for EmptyEnvVars { type Item = (String, String); fn next(&mut self) -> Option<Self::Item> { None } }
pub fn vars() -> EmptyEnvVars { EmptyEnvVars }

pub fn current_dir() -> core::io::Result<super::path::PathBuf> {
    let mut buf = [0u8; 256];
    let n = user::syscall::raw::getcwd(buf.as_mut_ptr(), buf.len());
    if n < 0 { Err(core::io::Error::last_os_error()) }
    else { Ok(super::path::PathBuf::from(&buf[..n as usize])) }
}

pub fn set_current_dir(_path: &super::path::Path) -> core::io::Result<()> {
    let path_str = _path.to_str().unwrap_or("");
    user::syscall::chdir(path_str).map_err(|_| core::io::Error::last_os_error())
}