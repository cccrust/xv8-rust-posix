use alloc::string::{String, ToString};

pub struct EnvArgs {
    inner: xv8_libc::args::Args,
    current: usize,
}

impl EnvArgs {
    fn new() -> Self {
        EnvArgs { inner: unsafe { xv8_libc::args::Args::from_stack() }, current: 1 }
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

pub struct EmptyEnvVars;
impl Iterator for EmptyEnvVars {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> { None }
}

pub fn vars() -> EmptyEnvVars { EmptyEnvVars }

pub fn current_dir() -> super::io::Result<super::path::PathBuf> {
    Err(super::io::ErrorKind::Unsupported.into())
}

pub fn set_current_dir(path: &super::path::Path) -> super::io::Result<()> {
    let path_str = path.to_str().unwrap_or("");
    let n = xv8_libc::chdir(path_str.as_ptr());
    if n < 0 { Err(super::io::ErrorKind::Other.into()) } else { Ok(()) }
}
