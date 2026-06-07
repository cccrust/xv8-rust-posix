use core::time::Duration;

use crate::io;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};

pub const TFD_NONBLOCK: u32 = 0x0800;
pub const TFD_CLOEXEC: u32 = 0x20000;
pub const TFD_TIMER_ABSTIME: u32 = 0x0001;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ItimerSpec {
    pub it_interval: Duration,
    pub it_value: Duration,
}

pub struct TimerFd(pub OwnedFd);

impl TimerFd {
    pub fn new(clockid: i32, flags: u32) -> io::Result<Self> {
        let ret = unsafe { xv8_libc::timerfd_create(clockid, flags) };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(TimerFd(unsafe { OwnedFd::from_raw_fd(ret as RawFd) }))
        }
    }

    pub fn set_time(
        &self,
        flags: u32,
        new: &ItimerSpec,
        old: Option<&mut ItimerSpec>,
    ) -> io::Result<()> {
        let ret = unsafe {
            xv8_libc::timerfd_settime(
                self.0.as_raw_fd() as usize,
                flags,
                new as *const _ as usize,
                old.map_or(0, |o| o as *mut _ as usize),
            )
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn get_time(&self) -> io::Result<ItimerSpec> {
        let mut curr: ItimerSpec = ItimerSpec {
            it_interval: Duration::ZERO,
            it_value: Duration::ZERO,
        };
        let ret = unsafe {
            xv8_libc::timerfd_gettime(self.0.as_raw_fd() as usize, &mut curr as *mut _ as usize)
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(curr)
        }
    }
}

impl AsRawFd for TimerFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsFd for TimerFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
