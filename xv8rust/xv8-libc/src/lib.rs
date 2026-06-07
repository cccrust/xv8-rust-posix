#![no_std]

pub mod raw;
pub mod args;

pub use raw::{read, write, open, close, lseek, fstat, exit, getpid, chdir, sbrk, Stat, isatty, mkdir};
pub use raw::{fork, exec, dup, dup2, pipe, wait, readlink, getenv, setenv, unsetenv, clearenv};
pub use raw::{socket, send, receive, ioctl, tcgetattr, tcsetattr};
pub use raw::{unlink, link, rename, chmod, fchmod, chown, fchown, access, symlink, truncate, ftruncate, getuid, getgid};
pub use raw::{
    tcp_socket, tcp_bind, tcp_listen, tcp_accept, tcp_connect, tcp_send, tcp_recv,
};
pub use raw::{sleep, uptime, time, nanosleep, clock_gettime, clock_getres, clock_settime};
pub use raw::{fcntl, poll, epoll_create1, epoll_ctl, epoll_wait, PollFd, EpollEvent,
    POLLIN, POLLOUT, POLLERR, POLLHUP,
    EPOLLIN, EPOLLOUT, EPOLLERR, EPOLLHUP, EPOLLRDHUP,
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD,
    F_GETFL, F_SETFL, O_NONBLOCK, EAGAIN};
pub use raw::{clone, clone_tls, gettid, exit_group, futex, FUTEX_WAIT, FUTEX_WAKE};
pub use raw::{eventfd2, EFD_SEMAPHORE, EFD_NONBLOCK, EFD_CLOEXEC};
pub use raw::{timerfd_create, timerfd_settime, timerfd_gettime,
    TFD_NONBLOCK, TFD_CLOEXEC, TFD_TIMER_ABSTIME,
    CLOCK_REALTIME, CLOCK_MONOTONIC};
pub use raw::{memfd_create, pidfd_open};
pub use raw::MFD_CLOEXEC;


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

pub struct IoctlCmd;

impl IoctlCmd {
    pub const CONSOLE_SET_RAW: usize = 1;
    pub const CONSOLE_SET_FG_PID: usize = 2;
    pub const SOCKET_GET_PORT: usize = 3;
}

pub const TCSANOW: usize = 0;

/// POSIX open flags matching the kernel's OpenFlag constants
pub struct OpenFlag;

impl OpenFlag {
    pub const READ_ONLY: usize = 0x000;
    pub const WRITE_ONLY: usize = 0x001;
    pub const READ_WRITE: usize = 0x002;
    pub const CREATE: usize = 0x040;
    pub const EXCLUSIVE: usize = 0x080;
    pub const TRUNCATE: usize = 0x200;
    pub const APPEND: usize = 0x400;
    pub const NON_BLOCK: usize = 0x800;
}
