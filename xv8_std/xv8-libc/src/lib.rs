#![no_std]

pub mod raw;

pub use raw::{read, write, open, close, lseek, fstat, exit, getpid, Stat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fd(usize);

impl Fd {
    pub const STDIN: Fd = Fd(0);
    pub const STDOUT: Fd = Fd(1);
    pub const STDERR: Fd = Fd(2);

    pub fn as_raw(&self) -> usize {
        self.0
    }
}

impl core::fmt::Display for Fd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysError(u16);

impl SysError {
    pub fn from_code(code: u16) -> Self {
        SysError(code)
    }
    pub fn code(&self) -> u16 {
        self.0
    }
}

impl core::fmt::Display for SysError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SysError({})", self.0)
    }
}

#[inline(always)]
pub fn check(ret: isize) -> Result<usize, SysError> {
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(SysError::from_code((-ret) as u16))
    }
}

#[inline(always)]
pub fn check_unit(ret: isize) -> Result<(), SysError> {
    check(ret).map(|_| ())
}

pub fn strlen(s: *const u8) -> usize {
    let mut len = 0usize;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}