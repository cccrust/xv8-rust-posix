

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Fork = 1,
    Read = 5,
    Exec = 7,
    Write = 16,
    Open = 15,
    Close = 21,
    Dup = 10,
    Dup2 = 54,
    Pipe = 4,
    Lseek = 28,
    Fstat = 8,
    Exit = 2,
    Wait = 3,
    Getpid = 11,
    Chdir = 9,
    Mkdir = 20,
    Readlink = 45,
    Getenv = 103,
    Setenv = 104,
    Unsetenv = 105,
    Clearenv = 106,
    Sbrk = 12,
    Isatty = 59,
    Tcgetattr = 60,
    Tcsetattr = 61,
    Unlink = 18,
    Link = 19,
    Chmod = 31,
    Fchmod = 32,
    Chown = 33,
    Fchown = 34,
    Access = 35,
    Rename = 36,
    Symlink = 44,
    Truncate = 29,
    Ftruncate = 30,
    Mknod = 17,
    Ioctl = 23,
    Kill = 6,
    Sleep = 13,
    Uptime = 14,
    Getuid = 38,
    Geteuid = 39,
    Getgid = 40,
    Getegid = 41,
    Umask = 37,
    Socket = 24,
    Send = 25,
    Receive = 26,
    Time = 66,
    Nanosleep = 67,
    ClockGetTime = 68,
    ClockGetRes = 69,
    ClockSetTime = 70,
    TcpSocket = 108,
    TcpBind = 109,
    TcpListen = 110,
    TcpAccept = 111,
    TcpConnect = 112,
    TcpSend = 113,
    TcpRecv = 114,
    Fcntl = 115,
    Poll = 116,
    EpollCreate1 = 117,
    EpollCtl = 118,
    EpollWait = 119,
    Clone = 120,
    Gettid = 121,
    ExitGroup = 122,
    Futex = 123,
    EventFd2 = 124,
    TimerFdCreate = 126,
    TimerFdSettime = 127,
    TimerFdGettime = 128,
    MemFdCreate = 129,
    PidFdOpen = 130,
    SetNs = 140,
    Unshare = 141,
    Sethostname = 146,
    Gethostname = 147,
    CapGet = 142,
    CapSet = 143,
    Seccomp = 144,
    OverlayMount = 148,
    OverlayUmount = 149,
}

#[inline(always)]
fn syscall1(syscall: Syscall, a0: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            inlateout("a0") a0 as isize => ret,
        );
    }
    ret
}

#[inline(always)]
fn syscall2(syscall: Syscall, a0: usize, a1: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            inlateout("a0") a0 as isize => ret,
            in("a1") a1,
        );
    }
    ret
}

#[inline(always)]
fn syscall3(syscall: Syscall, a0: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            inlateout("a0") a0 as isize => ret,
            in("a1") a1,
            in("a2") a2,
        );
    }
    ret
}

#[inline(always)]
fn syscall4(syscall: Syscall, a0: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            inlateout("a0") a0 as isize => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
        );
    }
    ret
}

#[inline(always)]
fn syscall5(syscall: Syscall, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            inlateout("a0") a0 as isize => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            in("a4") a4,
        );
    }
    ret
}

pub fn socket(port: u16) -> isize {
    syscall1(Syscall::Socket, port as usize)
}

pub fn tcp_socket() -> isize {
    syscall1(Syscall::TcpSocket, 0)
}

pub fn tcp_bind(fd: usize, port: u16) -> isize {
    syscall2(Syscall::TcpBind, fd, port as usize)
}

pub fn tcp_listen(fd: usize) -> isize {
    syscall1(Syscall::TcpListen, fd)
}

pub fn tcp_accept(fd: usize) -> isize {
    syscall1(Syscall::TcpAccept, fd)
}

pub fn tcp_connect(fd: usize, dest_ip: *const u8, dest_port: u16) -> isize {
    syscall3(Syscall::TcpConnect, fd, dest_ip as usize, dest_port as usize)
}

pub fn tcp_send(fd: usize, buf: *const u8, len: usize) -> isize {
    syscall3(Syscall::TcpSend, fd, buf as usize, len)
}

pub fn tcp_recv(fd: usize, buf: *mut u8, len: usize) -> isize {
    syscall3(Syscall::TcpRecv, fd, buf as usize, len)
}

pub fn send(fd: usize, buf: *const u8, len: usize, dest_ip: *const u8, dest_port: u16) -> isize {
    syscall5(
        Syscall::Send,
        fd,
        buf as usize,
        len,
        dest_ip as usize,
        dest_port as usize,
    )
}

pub fn receive(
    fd: usize,
    buf: *mut u8,
    len: usize,
    src_ip: *mut u8,
    src_port: *mut u16,
) -> isize {
    syscall5(
        Syscall::Receive,
        fd,
        buf as usize,
        len,
        src_ip as usize,
        src_port as usize,
    )
}

pub fn read(fd: usize, buf: *mut u8, len: usize) -> isize {
    syscall3(Syscall::Read, fd, buf as usize, len)
}

pub fn fork() -> isize {
    syscall1(Syscall::Fork, 0)
}

pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
    syscall3(Syscall::Write, fd, buf as usize, len)
}

pub fn exec(path: *const u8, argv: *const *const u8) -> isize {
    syscall2(Syscall::Exec, path as usize, argv as usize)
}

pub fn open(path: *const u8, flags: usize) -> isize {
    syscall2(Syscall::Open, path as usize, flags)
}

pub fn close(fd: usize) -> isize {
    syscall1(Syscall::Close, fd)
}

pub fn dup(fd: usize) -> isize {
    syscall1(Syscall::Dup, fd)
}

pub fn dup2(oldfd: usize, newfd: usize) -> isize {
    syscall2(Syscall::Dup2, oldfd, newfd)
}

pub fn pipe(fds: *mut usize) -> isize {
    syscall1(Syscall::Pipe, fds as usize)
}

pub fn wait(status: *mut usize) -> isize {
    syscall1(Syscall::Wait, status as usize)
}

pub fn lseek(fd: usize, offset: isize, whence: i32) -> isize {
    syscall3(Syscall::Lseek, fd, offset as usize, whence as usize)
}

pub fn fstat(fd: usize, stat: *mut Stat) -> isize {
    syscall2(Syscall::Fstat, fd, stat as usize)
}

pub fn exit(code: usize) -> ! {
    syscall1(Syscall::Exit, code);
    unsafe { core::hint::unreachable_unchecked() }
}

pub fn getpid() -> isize {
    syscall1(Syscall::Getpid, 0)
}

pub fn chdir(path: *const u8) -> isize {
    syscall1(Syscall::Chdir, path as usize)
}

pub fn mkdir(path: *const u8, mode: usize) -> isize {
    syscall2(Syscall::Mkdir, path as usize, mode)
}

pub fn readlink(path: *const u8, buf: *mut u8, len: usize) -> isize {
    syscall3(Syscall::Readlink, path as usize, buf as usize, len)
}

pub fn getenv(name: *const u8, buf: *mut u8, len: usize) -> isize {
    syscall3(Syscall::Getenv, name as usize, buf as usize, len)
}

pub fn sleep(ticks: usize) -> isize {
    syscall1(Syscall::Sleep, ticks)
}

pub fn uptime() -> isize {
    syscall1(Syscall::Uptime, 0)
}

pub fn setenv(name: *const u8, value: *const u8, overwrite: isize) -> isize {
    syscall3(Syscall::Setenv, name as usize, value as usize, overwrite as usize)
}

pub fn unsetenv(name: *const u8) -> isize {
    syscall1(Syscall::Unsetenv, name as usize)
}

pub fn clearenv() -> isize {
    syscall1(Syscall::Clearenv, 0)
}

pub fn sbrk(n: isize) -> isize {
    syscall1(Syscall::Sbrk, n as usize)
}

pub fn isatty(fd: usize) -> isize {
    syscall1(Syscall::Isatty, fd)
}

pub fn ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
    syscall3(Syscall::Ioctl, fd, cmd, arg)
}

pub fn tcgetattr(fd: usize, termios: *mut usize) -> isize {
    syscall2(Syscall::Tcgetattr, fd, termios as usize)
}

pub fn tcsetattr(fd: usize, action: usize, termios: *const usize) -> isize {
    syscall3(Syscall::Tcsetattr, fd, action, termios as usize)
}

pub fn unlink(path: *const u8) -> isize {
    syscall1(Syscall::Unlink, path as usize)
}

pub fn link(old: *const u8, new: *const u8) -> isize {
    syscall2(Syscall::Link, old as usize, new as usize)
}

pub fn rename(old: *const u8, new: *const u8) -> isize {
    syscall2(Syscall::Rename, old as usize, new as usize)
}

pub fn chmod(path: *const u8, mode: usize) -> isize {
    syscall2(Syscall::Chmod, path as usize, mode)
}

pub fn fchmod(fd: usize, mode: usize) -> isize {
    syscall2(Syscall::Fchmod, fd, mode)
}

pub fn chown(path: *const u8, uid: usize, gid: usize) -> isize {
    syscall3(Syscall::Chown, path as usize, uid, gid)
}

pub fn fchown(fd: usize, uid: usize, gid: usize) -> isize {
    syscall3(Syscall::Fchown, fd, uid, gid)
}

pub fn access(path: *const u8, mode: usize) -> isize {
    syscall2(Syscall::Access, path as usize, mode)
}

pub fn symlink(target: *const u8, linkpath: *const u8) -> isize {
    syscall2(Syscall::Symlink, target as usize, linkpath as usize)
}

pub fn truncate(path: *const u8, length: usize) -> isize {
    syscall2(Syscall::Truncate, path as usize, length)
}

pub fn ftruncate(fd: usize, length: usize) -> isize {
    syscall2(Syscall::Ftruncate, fd, length)
}

pub fn getuid() -> isize {
    syscall1(Syscall::Getuid, 0)
}

pub fn getgid() -> isize {
    syscall1(Syscall::Getgid, 0)
}

pub fn time(t: *mut u32) -> isize {
    syscall1(Syscall::Time, t as usize)
}

pub fn nanosleep(req: *const u8, rem: *mut u8) -> isize {
    syscall2(Syscall::Nanosleep, req as usize, rem as usize)
}

pub fn clock_gettime(clock_id: usize, ts: *mut u8) -> isize {
    syscall2(Syscall::ClockGetTime, clock_id, ts as usize)
}

pub fn clock_getres(clock_id: usize, ts: *mut u8) -> isize {
    syscall2(Syscall::ClockGetRes, clock_id, ts as usize)
}

pub fn clock_settime(clock_id: usize, ts: *const u8) -> isize {
    syscall2(Syscall::ClockSetTime, clock_id, ts as usize)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub dev: u32,
    pub ino: u32,
    pub r#type: u16,
    pub mode: u16,
    pub nlink: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u64,
    pub atime: u32,
    pub mtime: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

pub const POLLIN: i16 = 0x001;
pub const POLLOUT: i16 = 0x004;
pub const POLLERR: i16 = 0x008;
pub const POLLHUP: i16 = 0x010;

pub const EPOLLIN: u32 = 0x001;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
pub const EPOLLRDHUP: u32 = 0x2000;

pub const EPOLL_CTL_ADD: usize = 1;
pub const EPOLL_CTL_DEL: usize = 2;
pub const EPOLL_CTL_MOD: usize = 3;

pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> isize {
    syscall3(Syscall::Fcntl, fd, cmd, arg)
}

pub fn poll(fds: *mut PollFd, nfds: usize, timeout: isize) -> isize {
    syscall3(Syscall::Poll, fds as usize, nfds, timeout as usize)
}

pub fn epoll_create1(flags: usize) -> isize {
    syscall1(Syscall::EpollCreate1, flags)
}

pub fn epoll_ctl(epfd: usize, op: usize, fd: usize, event: *const EpollEvent) -> isize {
    syscall4(Syscall::EpollCtl, epfd, op, fd, event as usize)
}

pub fn epoll_wait(epfd: usize, events: *mut EpollEvent, max_events: usize, timeout: isize) -> isize {
    syscall4(Syscall::EpollWait, epfd, events as usize, max_events, timeout as usize)
}

pub fn clone(flags: usize, stack: usize) -> isize {
    syscall2(Syscall::Clone, flags, stack)
}

pub fn clone_tls(flags: usize, stack: usize, ptid: usize, tls: usize) -> isize {
    syscall4(Syscall::Clone, flags, stack, ptid, tls)
}

pub fn gettid() -> isize {
    syscall1(Syscall::Gettid, 0)
}

pub fn exit_group(code: usize) -> ! {
    syscall1(Syscall::ExitGroup, code);
    unsafe { core::hint::unreachable_unchecked() }
}

pub const F_GETFL: usize = 3;
pub const F_SETFL: usize = 4;
pub const O_NONBLOCK: usize = 0x800;

pub const EAGAIN: u16 = 11;

pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;

pub fn futex(uaddr: *const u32, op: u32, val: u32) -> isize {
    syscall3(Syscall::Futex, uaddr as usize, op as usize, val as usize)
}

pub const EFD_SEMAPHORE: u32 = 0x0001;
pub const EFD_NONBLOCK: u32 = 0x0800;
pub const EFD_CLOEXEC: u32 = 0x20000;

pub const TFD_NONBLOCK: u32 = 0x0800;
pub const TFD_CLOEXEC: u32 = 0x20000;
pub const TFD_TIMER_ABSTIME: u32 = 0x0001;

pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;

pub const MFD_CLOEXEC: u32 = 0x0001;

pub fn eventfd2(initval: u32, flags: u32) -> isize {
    syscall2(Syscall::EventFd2, initval as usize, flags as usize)
}

pub fn timerfd_create(clockid: i32, flags: u32) -> isize {
    syscall2(Syscall::TimerFdCreate, clockid as usize, flags as usize)
}

pub fn timerfd_settime(fd: usize, flags: u32, new_val: usize, old_val: usize) -> isize {
    syscall4(Syscall::TimerFdSettime, fd, flags as usize, new_val, old_val)
}

pub fn timerfd_gettime(fd: usize, curr_val: usize) -> isize {
    syscall2(Syscall::TimerFdGettime, fd, curr_val)
}

pub fn memfd_create(name: usize, flags: u32) -> isize {
    syscall2(Syscall::MemFdCreate, name, flags as usize)
}

pub fn pidfd_open(pid: usize, flags: u32) -> isize {
    syscall2(Syscall::PidFdOpen, pid, flags as usize)
}

pub fn setns(fd: usize, nstype: u32) -> isize {
    syscall2(Syscall::SetNs, fd, nstype as usize)
}

pub fn unshare(flags: usize) -> isize {
    syscall1(Syscall::Unshare, flags)
}

pub fn sethostname(name: *const u8, len: usize) -> isize {
    syscall2(Syscall::Sethostname, name as usize, len)
}

pub fn gethostname(buf: *mut u8, len: usize) -> isize {
    syscall2(Syscall::Gethostname, buf as usize, len)
}

pub fn capget(hdr: *const usize, data: *mut usize) -> isize {
    syscall2(Syscall::CapGet, hdr as usize, data as usize)
}

pub fn capset(hdr: *const usize, data: *const usize) -> isize {
    syscall2(Syscall::CapSet, hdr as usize, data as usize)
}

pub fn seccomp(op: usize, flags: usize, args: *const u8) -> isize {
    syscall3(Syscall::Seccomp, op, flags, args as usize)
}

pub fn overlay_mount(mount_point: *const u8, upper: *const u8, lower: *const u8) -> isize {
    syscall3(Syscall::OverlayMount, mount_point as usize, upper as usize, lower as usize)
}

pub fn overlay_umount(mount_point: *const u8) -> isize {
    syscall1(Syscall::OverlayUmount, mount_point as usize)
}
