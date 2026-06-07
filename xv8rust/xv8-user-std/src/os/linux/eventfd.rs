use crate::io;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

pub const EFD_SEMAPHORE: u32 = 0x0001;
pub const EFD_NONBLOCK: u32 = 0x0800;
pub const EFD_CLOEXEC: u32 = 0x20000;

pub struct EventFd(pub OwnedFd);

impl EventFd {
    pub fn new(initval: u32, flags: u32) -> io::Result<Self> {
        let ret = unsafe { xv8_libc::eventfd2(initval, flags) };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(EventFd(unsafe { OwnedFd::from_raw_fd(ret as RawFd) }))
        }
    }

    pub fn read(&self) -> io::Result<u64> {
        let mut val: u64 = 0;
        let buf = unsafe {
            core::slice::from_raw_parts_mut(&mut val as *mut _ as *mut u8, 8)
        };
        let n = xv8_libc::read(self.0.as_raw_fd() as usize, buf.as_mut_ptr(), 8);
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(val)
        }
    }

    pub fn write(&self, val: u64) -> io::Result<()> {
        let buf = unsafe {
            core::slice::from_raw_parts(&val as *const _ as *const u8, 8)
        };
        let n = xv8_libc::write(self.0.as_raw_fd() as usize, buf.as_ptr(), 8);
        if n < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl AsRawFd for EventFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsFd for EventFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
