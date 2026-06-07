use crate::os::fd::{FromRawFd, IntoRawFd, OwnedFd, RawFd};

pub fn memfd_create(flags: u32) -> Result<OwnedFd, core::ffi::c_int> {
    let ret = xv8_libc::memfd_create(0, flags);
    if ret < 0 {
        Err((-ret) as core::ffi::c_int)
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(ret as RawFd) })
    }
}
