use alloc::vec::Vec;
use crate::io::{self, Read, Write};
use crate::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use xv8_libc;

#[derive(Debug)]
pub struct UnixStream {
    fd: usize,
}

impl UnixStream {
    pub fn connect<P: AsRef<crate::path::Path>>(_path: P) -> io::Result<Self> {
        Err(io::ErrorKind::Unsupported.into())
    }

    pub fn pair() -> io::Result<(UnixStream, UnixStream)> {
        let mut fds = [0usize; 2];
        let ret = xv8_libc::pipe(fds.as_mut_ptr());
        if ret < 0 {
            Err(io::ErrorKind::Other.into())
        } else {
            Ok((UnixStream { fd: fds[0] }, UnixStream { fd: fds[1] }))
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let fd = xv8_libc::dup(self.fd);
        if fd < 0 {
            Err(io::ErrorKind::Other.into())
        } else {
            Ok(UnixStream { fd: fd as usize })
        }
    }

    pub fn fd(&self) -> usize { self.fd }
}

impl Read for UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = xv8_libc::read(self.fd, buf.as_mut_ptr(), buf.len());
        if n < 0 { Err(io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
}

impl Write for UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = xv8_libc::write(self.fd, buf.as_ptr(), buf.len());
        if n < 0 { Err(io::ErrorKind::Other.into()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl AsRawFd for UnixStream {
    fn as_raw_fd(&self) -> RawFd { self.fd as i32 }
}

impl IntoRawFd for UnixStream {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd as i32;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for UnixStream {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        UnixStream { fd: fd as usize }
    }
}

impl Drop for UnixStream {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}

pub struct UnixListener {
    fd: usize,
}

impl UnixListener {
    pub fn bind<P: AsRef<crate::path::Path>>(_path: P) -> io::Result<Self> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

pub struct UnixDatagram;

impl UnixDatagram {
    pub fn pair() -> io::Result<(UnixDatagram, UnixDatagram)> {
        Err(io::ErrorKind::Unsupported.into())
    }
}
