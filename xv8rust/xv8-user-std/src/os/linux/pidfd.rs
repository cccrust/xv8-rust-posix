use crate::os::fd::{FromRawFd, IntoRawFd, OwnedFd, RawFd};

pub fn pidfd_open(pid: u32, flags: u32) -> Result<OwnedFd, core::ffi::c_int> {
    let ret = xv8_libc::pidfd_open(pid as usize, flags);
    if ret < 0 {
        Err((-ret) as core::ffi::c_int)
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(ret as RawFd) })
    }
}
