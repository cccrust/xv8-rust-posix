use core::marker::PhantomData;

use xv8_libc;

pub type RawFd = i32;

pub trait AsRawFd {
    fn as_raw_fd(&self) -> i32;
}

pub trait AsFd {
    fn as_fd(&self) -> BorrowedFd<'_>;
}

pub trait IntoRawFd {
    fn into_raw_fd(self) -> i32;
}

pub trait FromRawFd {
    unsafe fn from_raw_fd(fd: i32) -> Self;
}

#[derive(Copy, Clone)]
pub struct BorrowedFd<'a> {
    fd: RawFd,
    _marker: PhantomData<&'a RawFd>,
}

impl<'a> BorrowedFd<'a> {
    pub unsafe fn borrow_raw(fd: RawFd) -> Self {
        Self { fd, _marker: PhantomData }
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

#[derive(Debug)]
pub struct OwnedFd {
    fd: RawFd,
}

impl OwnedFd {
    pub unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }

    pub fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        core::mem::forget(self);
        fd
    }
}

impl AsFd for OwnedFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl IntoRawFd for OwnedFd {
    fn into_raw_fd(self) -> i32 {
        let fd = self.fd;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for OwnedFd {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        Self { fd }
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        if self.fd >= 0 {
            let _ = xv8_libc::close(self.fd as usize);
        }
    }
}

impl AsRawFd for BorrowedFd<'_> {
    fn as_raw_fd(&self) -> i32 { self.fd }
}

impl AsRawFd for OwnedFd {
    fn as_raw_fd(&self) -> i32 { self.fd }
}

impl AsFd for crate::io::Stdin {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl AsFd for crate::io::Stdout {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl AsFd for crate::io::Stderr {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl AsRawFd for crate::io::Stdin {
    fn as_raw_fd(&self) -> i32 { 0 }
}

impl AsRawFd for crate::io::Stdout {
    fn as_raw_fd(&self) -> i32 { 1 }
}

impl AsRawFd for crate::io::Stderr {
    fn as_raw_fd(&self) -> i32 { 2 }
}

impl AsRawFd for crate::fs::File {
    fn as_raw_fd(&self) -> i32 { self.as_raw_fd() as i32 }
}

impl IntoRawFd for crate::fs::File {
    fn into_raw_fd(self) -> i32 {
        let fd = self.as_raw_fd() as i32;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for crate::fs::File {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        crate::fs::File::from_raw_fd(fd as usize)
    }
}
