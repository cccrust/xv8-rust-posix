pub fn exit(code: i32) -> ! {
    xv8_libc::exit(code as usize)
}

pub struct Command { _prog: alloc::string::String }
impl Command {
    pub fn new(prog: &str) -> Self { Command { _prog: prog.into() } }
    pub fn arg(&mut self, _arg: &str) -> &mut Self { self }
    pub fn args<I: IntoIterator<Item = alloc::string::String>>(&mut self, _args: I) -> &mut Self { self }
    pub fn spawn(&mut self) -> super::io::Result<Child> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
    pub fn output(&mut self) -> super::io::Result<ExitStatus> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
}

pub struct Child { _pid: usize }
impl Child {
    pub fn id(&self) -> usize { self._pid }
    pub fn wait(&mut self) -> super::io::Result<ExitStatus> {
        Err(super::io::ErrorKind::Unsupported.into())
    }
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
