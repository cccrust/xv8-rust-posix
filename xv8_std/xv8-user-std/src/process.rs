pub fn exit(code: i32) -> ! {
    xv8_libc::exit(code as usize)
}

pub fn id() -> u32 {
    xv8_libc::getpid() as u32
}

pub struct Command {
    _prog: alloc::string::String,
    _args: alloc::vec::Vec<alloc::string::String>,
}
impl Command {
    pub fn new(prog: &str) -> Self {
        Command { _prog: prog.into(), _args: alloc::vec::Vec::new() }
    }
    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self {
        self._args.push(arg.as_ref().into());
        self
    }
    pub fn args<I: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, args: I) -> &mut Self {
        for a in args { self._args.push(a.as_ref().into()); }
        self
    }
    pub fn env_clear(&mut self) -> &mut Self { self }
    pub fn env<K: AsRef<str>, V: AsRef<str>>(&mut self, _key: K, _val: V) -> &mut Self { self }
    pub fn env_remove<K: AsRef<str>>(&mut self, _key: K) -> &mut Self { self }
    pub fn envs<I: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: AsRef<str>>(&mut self, _envs: I) -> &mut Self { self }
    pub fn status(&mut self) -> super::io::Result<ExitStatus> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
    pub fn stdin<T: Into<Stdio>>(&mut self, _cfg: T) -> &mut Self { self }
    pub fn stdout<T: Into<Stdio>>(&mut self, _cfg: T) -> &mut Self { self }
    pub fn stderr<T: Into<Stdio>>(&mut self, _cfg: T) -> &mut Self { self }
    pub fn spawn(&mut self) -> super::io::Result<Child> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
    pub fn output(&mut self) -> super::io::Result<Output> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
}

pub struct Child {
    pub stdin: Option<super::io::Stdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<super::io::Stderr>,
    _pid: usize,
}
impl Child {
    pub fn id(&self) -> usize { self._pid }
    pub fn wait(&mut self) -> super::io::Result<ExitStatus> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
    pub fn wait_with_output(self) -> super::io::Result<Output> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
}

pub struct ChildStdout;
impl super::io::Read for ChildStdout {
    fn read(&mut self, _buf: &mut [u8]) -> super::io::Result<usize> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
}

pub struct Output {
    pub status: ExitStatus,
    pub stdout: alloc::vec::Vec<u8>,
    pub stderr: alloc::vec::Vec<u8>,
}

pub struct ExitStatus;
impl ExitStatus {
    pub fn code(&self) -> Option<i32> { None }
    pub fn success(&self) -> bool { true }
}

pub enum Stdio {
    Inherit,
    Null,
    Piped,
    Custom(super::io::Stdout),
}

impl Stdio {
    pub fn piped() -> Self { Stdio::Piped }
    pub fn null() -> Self { Stdio::Null }
    pub fn inherit() -> Self { Stdio::Inherit }
}

impl Default for Stdio {
    fn default() -> Self { Stdio::Inherit }
}

impl From<super::fs::File> for Stdio {
    fn from(_f: super::fs::File) -> Self { Stdio::Inherit }
}

impl From<ChildStdout> for Stdio {
    fn from(_c: ChildStdout) -> Self { Stdio::Inherit }
}

impl From<super::io::Stdin> for Stdio {
    fn from(_s: super::io::Stdin) -> Self { Stdio::Inherit }
}

impl From<super::io::Stdout> for Stdio {
    fn from(_s: super::io::Stdout) -> Self { Stdio::Inherit }
}
