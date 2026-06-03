use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: Option<&'static str>,
}

impl Error {
    pub fn new(kind: ErrorKind, message: &'static str) -> Self {
        Error { kind, message: Some(message) }
    }
    pub fn kind(&self) -> ErrorKind { self.kind.clone() }
    pub fn last_os_error() -> Self {
        Error { kind: ErrorKind::Other, message: Some("os error") }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error { kind, message: None }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.message {
            Some(message) => write!(f, "{}", message),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl crate::error::Error for Error {
    fn source(&self) -> Option<&(dyn crate::error::Error + 'static)> { None }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut tmp = [0u8; 1024];
        let mut total = 0;
        loop {
            let n = self.read(&mut tmp)?;
            if n == 0 { break; }
            buf.extend_from_slice(&tmp[..n]);
            total += n;
        }
        Ok(total)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        let mut bytes = Vec::new();
        let n = self.read_to_end(&mut bytes)?;
        let s = core::str::from_utf8(&bytes).map_err(|_| ErrorKind::InvalidData)?;
        buf.push_str(s);
        Ok(n)
    }

    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes { inner: self }
    }

    fn chain<R: Read>(self, other: R) -> Chain<Self, R> where Self: Sized {
        Chain { first: self, second: other, done_first: false }
    }

    fn take(self, limit: u64) -> Take<Self> where Self: Sized {
        Take { inner: self, limit }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.read(&mut buf[offset..])?;
            if n == 0 { return Err(ErrorKind::UnexpectedEof.into()); }
            offset += n;
        }
        Ok(())
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
    fn by_ref(&mut self) -> &mut Self where Self: Sized { self }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.write(&buf[offset..])?;
            if n == 0 { return Err(ErrorKind::WriteZero.into()); }
            offset += n;
        }
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> Result<()> {
        struct Adapter<'a, T: Write + ?Sized>(&'a mut T);
        impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
            }
        }
        match core::fmt::write(&mut Adapter(self), args) {
            Ok(()) => Ok(()),
            Err(_) => Err(ErrorKind::Other.into()),
        }
    }
}

pub trait BufRead: Read {
    fn fill_buf(&mut self) -> Result<&[u8]>;
    fn consume(&mut self, amt: usize);

    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let mut total = 0;
        loop {
            let (slice, found_newline);
            let n;
            let to_consume;
            {
                let available = self.fill_buf()?;
                if available.is_empty() { break; }
                if let Some(pos) = available.iter().position(|&b| b == b'\n') {
                    slice = available[..=pos].to_vec();
                    n = slice.len();
                    to_consume = pos + 1;
                    found_newline = true;
                } else {
                    slice = available.to_vec();
                    n = slice.len();
                    to_consume = n;
                    found_newline = false;
                }
            }
            self.consume(to_consume);
            buf.push_str(core::str::from_utf8(&slice).map_err(|_| ErrorKind::InvalidData)?);
            total += n;
            if found_newline { break; }
        }
        Ok(total)
    }

    fn lines(self) -> Lines<Self> where Self: Sized {
        Lines { inner: self }
    }
}

impl<T: Read + ?Sized> Read for Box<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { (**self).read(buf) }
}

impl<T: Read + ?Sized> Read for &mut T {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { (**self).read(buf) }
}

impl<T: BufRead + ?Sized> BufRead for Box<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> { (**self).fill_buf() }
    fn consume(&mut self, amt: usize) { (**self).consume(amt) }
}

impl<T: Write + ?Sized> Write for Box<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> { (**self).write(buf) }
    fn flush(&mut self) -> Result<()> { (**self).flush() }
}

impl<T: Write + ?Sized> Write for &mut T {
    fn write(&mut self, buf: &[u8]) -> Result<usize> { (**self).write(buf) }
    fn flush(&mut self) -> Result<()> { (**self).flush() }
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

pub struct StdinLock {
    buf: [u8; 1024],
    pos: usize,
    len: usize,
}

impl StdinLock {
    fn fill_buf_inner(&mut self) -> Result<&[u8]> {
        if self.pos >= self.len {
            self.pos = 0;
            let n = xv8_libc::read(0, self.buf.as_mut_ptr(), self.buf.len());
            if n <= 0 {
                self.len = 0;
                Ok(&[])
            } else {
                self.len = n as usize;
                Ok(&self.buf[..self.len])
            }
        } else {
            Ok(&self.buf[self.pos..self.len])
        }
    }
}

impl Read for StdinLock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.pos >= self.len {
            let n = xv8_libc::read(0, buf.as_mut_ptr(), buf.len());
            if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
        } else {
            let available = self.len - self.pos;
            let to_read = core::cmp::min(available, buf.len());
            buf[..to_read].copy_from_slice(&self.buf[self.pos..self.pos + to_read]);
            self.pos += to_read;
            Ok(to_read)
        }
    }
}

impl BufRead for StdinLock {
    fn fill_buf(&mut self) -> Result<&[u8]> { self.fill_buf_inner() }
    fn consume(&mut self, amt: usize) { self.pos = core::cmp::min(self.pos + amt, self.len); }
}

pub struct StdoutLock;

impl Write for StdoutLock {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = xv8_libc::write(1, buf.as_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

pub struct StderrLock;

impl Write for StderrLock {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = xv8_libc::write(2, buf.as_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl Stdin {
    pub fn lock(&self) -> StdinLock {
        StdinLock { buf: [0u8; 1024], pos: 0, len: 0 }
    }
    pub fn read_line(&self, buf: &mut String) -> Result<usize> {
        self.lock().read_line(buf)
    }
    pub fn lines(self) -> Lines<StdinLock> {
        Lines { inner: self.lock() }
    }
    pub fn is_terminal(&self) -> bool { xv8_libc::isatty(0) == 1 }
}

impl Write for Stdin {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = xv8_libc::write(0, buf.as_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = xv8_libc::read(0, buf.as_mut_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
}

impl Stdout {
    pub fn lock(&self) -> StdoutLock { StdoutLock }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = xv8_libc::write(1, buf.as_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl Stderr {
    pub fn lock(&self) -> StderrLock { StderrLock }
}

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = xv8_libc::write(2, buf.as_ptr(), buf.len());
        if n < 0 { Err(ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

pub struct BufReader<R> {
    inner: R,
    buf: [u8; 8192],
    pos: usize,
    len: usize,
}

impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> Self {
        BufReader { inner, buf: [0u8; 8192], pos: 0, len: 0 }
    }
    pub fn into_inner(self) -> R { self.inner }
    pub fn get_ref(&self) -> &R { &self.inner }
    pub fn get_mut(&mut self) -> &mut R { &mut self.inner }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
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

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
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

pub struct Lines<B: BufRead> {
    inner: B,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        match self.inner.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => {
                if buf.ends_with('\n') { buf.pop(); }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Bytes<R: Read> {
    inner: R,
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut byte = [0u8; 1];
        match self.inner.read(&mut byte) {
            Ok(0) => None,
            Ok(_) => Some(Ok(byte[0])),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct Chain<T: Read, U: Read> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T: Read, U: Read> Read for Chain<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.done_first {
            match self.first.read(buf) {
                Ok(0) => self.done_first = true,
                Ok(n) => return Ok(n),
                Err(e) => return Err(e),
            }
        }
        self.second.read(buf)
    }
}

pub struct Take<T: Read> {
    inner: T,
    limit: u64,
}

impl<T: Read> Read for Take<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.limit == 0 { return Ok(0); }
        let max = core::cmp::min(buf.len() as u64, self.limit) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        self.limit -= n as u64;
        Ok(n)
    }
}

pub fn stdin() -> Stdin { Stdin }
pub fn stdout() -> Stdout { Stdout }
pub fn stderr() -> Stderr { Stderr }

pub trait IsTerminal {
    fn is_terminal(&self) -> bool;
}

impl IsTerminal for Stdin { fn is_terminal(&self) -> bool { xv8_libc::isatty(0) == 1 } }
impl IsTerminal for Stdout { fn is_terminal(&self) -> bool { xv8_libc::isatty(1) == 1 } }
impl IsTerminal for Stderr { fn is_terminal(&self) -> bool { xv8_libc::isatty(2) == 1 } }
impl IsTerminal for StdinLock { fn is_terminal(&self) -> bool { xv8_libc::isatty(0) == 1 } }
impl IsTerminal for StdoutLock { fn is_terminal(&self) -> bool { xv8_libc::isatty(1) == 1 } }
impl IsTerminal for StderrLock { fn is_terminal(&self) -> bool { xv8_libc::isatty(2) == 1 } }

pub fn _print(args: core::fmt::Arguments<'_>) {
    let _ = Stdout.write_fmt(args);
}

pub fn _eprint(args: core::fmt::Arguments<'_>) {
    let _ = Stderr.write_fmt(args);
}
