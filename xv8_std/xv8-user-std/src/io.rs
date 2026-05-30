use core::fmt;
use alloc::string::String;
use user::kernel::abi::SysError;
use user::syscall::{self, Fd};

fn syserr_to_io(err: SysError) -> core::io::Error {
    use core::io::ErrorKind;
    let kind = match err {
        SysError::NoEntry => ErrorKind::NotFound,
        SysError::NotPermitted => ErrorKind::PermissionDenied,
        SysError::BadDescriptor => ErrorKind::PermissionDenied,
        SysError::InvalidArgument => ErrorKind::InvalidInput,
        SysError::IoError => ErrorKind::Other,
        SysError::Interrupted => ErrorKind::Interrupted,
        _ => ErrorKind::Other,
    };
    kind.into()
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub fn lock(&self) -> StdinLock { StdinLock { pos: 0, len: 0 } }
}

impl core::io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> core::io::Result<usize> {
        syscall::read(Fd::STDIN, buf).map_err(syserr_to_io)
    }
}

impl fmt::Write for Stdin {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock { StdoutLock }
}

impl core::io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
        syscall::write(Fd::STDOUT, buf).map_err(syserr_to_io)
    }
    fn flush(&mut self) -> core::io::Result<()> { Ok(()) }
}

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

impl Stderr {
    pub fn lock(&self) -> StderrLock { StderrLock }
}

impl core::io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
        syscall::write(Fd::STDERR, buf).map_err(syserr_to_io)
    }
    fn flush(&mut self) -> core::io::Result<()> { Ok(()) }
}

impl fmt::Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub struct StdinLock {
    buf: [u8; 1024],
    pos: usize,
    len: usize,
}

impl StdinLock {
    fn fill_buf_internal(&mut self) -> core::io::Result<&[u8]> {
        if self.pos >= self.len {
            self.pos = 0;
            match syscall::read(Fd::STDIN, &mut self.buf) {
                Ok(0) => { self.len = 0; }
                Ok(n) => { self.len = n; }
                Err(e) => return Err(syserr_to_io(e)),
            }
        }
        Ok(&self.buf[self.pos..self.len])
    }
}

impl core::io::Read for StdinLock {
    fn read(&mut self, buf: &mut [u8]) -> core::io::Result<usize> {
        if self.pos >= self.len {
            self.pos = 0;
            match syscall::read(Fd::STDIN, &mut self.buf) {
                Ok(0) => return Ok(0),
                Ok(n) => { self.len = n; }
                Err(e) => return Err(syserr_to_io(e)),
            }
        }
        let available = self.len - self.pos;
        let to_read = core::cmp::min(available, buf.len());
        buf[..to_read].copy_from_slice(&self.buf[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl core::io::BufRead for StdinLock {
    fn fill_buf(&mut self) -> core::io::Result<&[u8]> {
        self.fill_buf_internal()
    }
    fn consume(&mut self, amt: usize) {
        self.pos = core::cmp::min(self.pos + amt, self.len);
    }
}

impl fmt::Write for StdinLock {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub struct StdoutLock;

impl core::io::Write for StdoutLock {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
        syscall::write(Fd::STDOUT, buf).map_err(syserr_to_io)
    }
    fn flush(&mut self) -> core::io::Result<()> { Ok(()) }
}

impl fmt::Write for StdoutLock {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub struct StderrLock;

impl core::io::Write for StderrLock {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
        syscall::write(Fd::STDERR, buf).map_err(syserr_to_io)
    }
    fn flush(&mut self) -> core::io::Result<()> { Ok(()) }
}

impl fmt::Write for StderrLock {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub struct BufReader<R> {
    inner: R,
    buf: [u8; 8192],
    pos: usize,
    len: usize,
}

impl<R: core::io::Read> BufReader<R> {
    pub fn new(inner: R) -> Self {
        BufReader { inner, buf: [0u8; 8192], pos: 0, len: 0 }
    }
    pub fn into_inner(self) -> R { self.inner }
    pub fn get_ref(&self) -> &R { &self.inner }
    pub fn get_mut(&mut self) -> &mut R { &mut self.inner }
}

impl<R: core::io::Read> core::io::Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> core::io::Result<usize> {
        if self.pos >= self.len {
            self.pos = 0;
            let n = self.inner.read(&mut self.buf)?;
            self.len = n;
            if n == 0 { return Ok(0); }
        }
        let available = self.len - self.pos;
        let to_read = core::cmp::min(available, buf.len());
        buf[..to_read].copy_from_slice(&self.buf[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl<R: core::io::Read> core::io::BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> core::io::Result<&[u8]> {
        if self.pos >= self.len {
            self.pos = 0;
            let n = self.inner.read(&mut self.buf)?;
            self.len = n;
            if n == 0 { return Ok(&[]); }
        }
        Ok(&self.buf[self.pos..self.len])
    }
    fn consume(&mut self, amt: usize) {
        self.pos = core::cmp::min(self.pos + amt, self.len);
    }
}

impl<R: core::io::Read> fmt::Write for BufReader<R> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub fn stdin() -> Stdin { Stdin }
pub fn stdout() -> Stdout { Stdout }
pub fn stderr() -> Stderr { Stderr }