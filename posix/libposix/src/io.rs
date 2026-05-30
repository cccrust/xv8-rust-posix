use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    InvalidInput,
    UnexpectedEof,
    BrokenPipe,
    Interrupted,
    Other,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: &'static str,
}

impl Error {
    pub const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Error { kind, message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            let n = self.write(remaining)?;
            remaining = &remaining[n..];
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<()>;
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        platform_read_stdin(buf)
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        platform_write_stdout(buf)
    }
    fn flush(&mut self) -> Result<()> {
        platform_flush_stdout()
    }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        platform_write_stderr(buf)
    }
    fn flush(&mut self) -> Result<()> {
        platform_flush_stderr()
    }
}

pub fn stdin() -> Stdin {
    Stdin
}
pub fn stdout() -> Stdout {
    Stdout
}
pub fn stderr() -> Stderr {
    Stderr
}

// Platform-specific implementations
#[cfg(unix)]
mod sys {
    use std::io::{self, Read, Write};

    pub fn read_stdin(buf: &mut [u8]) -> super::Result<usize> {
        io::stdin().read(buf).map_err(from_std_error)
    }
    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        io::stdout().write(buf).map_err(from_std_error)
    }
    pub fn flush_stdout() -> super::Result<()> {
        io::stdout().flush().map_err(from_std_error)
    }
    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        io::stderr().write(buf).map_err(from_std_error)
    }
    pub fn flush_stderr() -> super::Result<()> {
        io::stderr().flush().map_err(from_std_error)
    }
    fn from_std_error(e: io::Error) -> super::Error {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => super::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => super::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => super::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput => super::ErrorKind::InvalidInput,
            io::ErrorKind::UnexpectedEof => super::ErrorKind::UnexpectedEof,
            io::ErrorKind::BrokenPipe => super::ErrorKind::BrokenPipe,
            io::ErrorKind::Interrupted => super::ErrorKind::Interrupted,
            _ => super::ErrorKind::Other,
        };
        super::Error { kind, message: "I/O error" }
    }
}

#[cfg(windows)]
mod sys {
    use std::io::{self, Read, Write};

    pub fn read_stdin(buf: &mut [u8]) -> super::Result<usize> {
        io::stdin().read(buf).map_err(from_std_error)
    }
    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        io::stdout().write(buf).map_err(from_std_error)
    }
    pub fn flush_stdout() -> super::Result<()> {
        io::stdout().flush().map_err(from_std_error)
    }
    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        io::stderr().write(buf).map_err(from_std_error)
    }
    pub fn flush_stderr() -> super::Result<()> {
        io::stderr().flush().map_err(from_std_error)
    }
    fn from_std_error(e: io::Error) -> super::Error {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => super::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => super::ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => super::ErrorKind::AlreadyExists,
            io::ErrorKind::InvalidInput => super::ErrorKind::InvalidInput,
            io::ErrorKind::UnexpectedEof => super::ErrorKind::UnexpectedEof,
            io::ErrorKind::BrokenPipe => super::ErrorKind::BrokenPipe,
            io::ErrorKind::Interrupted => super::ErrorKind::Interrupted,
            _ => super::ErrorKind::Other,
        };
        super::Error { kind, message: "I/O error" }
    }
}

#[cfg(target_os = "none")]
mod sys {
    // xv8 platform — stub for now
    pub fn read_stdin(_buf: &mut [u8]) -> super::Result<usize> {
        Err(super::Error::new(super::ErrorKind::Other, "stdin not available"))
    }
    pub fn write_stdout(buf: &[u8]) -> super::Result<usize> {
        // On xv8, write to console via syscall
        // For now, stub
        let _ = buf;
        Ok(buf.len())
    }
    pub fn flush_stdout() -> super::Result<()> { Ok(()) }
    pub fn write_stderr(buf: &[u8]) -> super::Result<usize> {
        write_stdout(buf)
    }
    pub fn flush_stderr() -> super::Result<()> { Ok(()) }
}

fn platform_read_stdin(buf: &mut [u8]) -> Result<usize> {
    sys::read_stdin(buf)
}
fn platform_write_stdout(buf: &[u8]) -> Result<usize> {
    sys::write_stdout(buf)
}
fn platform_flush_stdout() -> Result<()> {
    sys::flush_stdout()
}
fn platform_write_stderr(buf: &[u8]) -> Result<usize> {
    sys::write_stderr(buf)
}
fn platform_flush_stderr() -> Result<()> {
    sys::flush_stderr()
}
