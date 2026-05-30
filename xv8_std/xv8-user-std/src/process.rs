pub fn exit(code: i32) -> ! {
    user::syscall::exit(code as usize)
}

pub struct Command { prog: alloc::string::String }
impl Command {
    pub fn new(_prog: &str) -> Self { Command { prog: _prog.into() } }
    pub fn arg(&mut self, _arg: &str) -> &mut Self { self }
    pub fn args<I: IntoIterator<Item = String>>(&mut self, _args: I) -> &mut Self { self }
    pub fn spawn(&mut self) -> core::io::Result<Child> {
        Err(core::io::Error::new(core::io::ErrorKind::Unsupported, "Command::spawn not implemented"))
    }
    pub fn output(&mut self) -> core::io::Result<ExitStatus> {
        Err(core::io::Error::new(core::io::ErrorKind::Unsupported, "Command::output not implemented"))
    }
}

pub struct Child { _pid: usize }
impl Child {
    pub fn id(&self) -> usize { self._pid }
    pub fn kill(&self) -> core::io::Result<()> {
        Err(core::io::Error::new(core::io::ErrorKind::Unsupported, "Child::kill not implemented"))
    }
    pub fn wait(&mut self) -> core::io::Result<ExitStatus> {
        Err(core::io::Error::new(core::io::ErrorKind::Unsupported, "Child::wait not implemented"))
    }
}

pub struct ExitStatus;
impl ExitStatus {
    pub fn code(&self) -> Option<i32> { None }
    pub fn success(&self) -> bool { true }
}

impl ExitStatus {
    pub fn new(_code: i32) -> Self { ExitStatus }
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