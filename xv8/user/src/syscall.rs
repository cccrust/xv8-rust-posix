pub mod raw {
    use core::arch::asm;

    use kernel::abi::{Stat, Syscall};

    #[inline(always)]
    fn syscall0(syscall: Syscall) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                lateout("a0") ret,
            );
        }
        ret
    }

    #[inline(always)]
    fn syscall1(syscall: Syscall, a0: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
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
            asm!(
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
            asm!(
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
            asm!(
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
            asm!(
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

    #[inline(always)]
    fn syscall6(syscall: Syscall, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") syscall as usize,
                inlateout("a0") a0 as isize => ret,
                in("a1") a1,
                in("a2") a2,
                in("a3") a3,
                in("a4") a4,
                in("a5") a5,
            );
        }
        ret
    }

    pub fn fork() -> isize {
        syscall0(Syscall::Fork)
    }

    pub fn exit(code: usize) -> ! {
        syscall1(Syscall::Exit, code);
        unreachable!();
    }

    pub fn wait(status: *mut usize) -> isize {
        syscall1(Syscall::Wait, status as usize)
    }

    pub fn pipe(fds: *mut usize) -> isize {
        syscall1(Syscall::Pipe, fds as usize)
    }

    pub fn mkfifo(path: *const u8) -> isize {
        syscall1(Syscall::Mkfifo, path as usize)
    }

    pub fn pipe2(fds: *mut usize, flags: usize) -> isize {
        syscall2(Syscall::Pipe2, fds as usize, flags)
    }

    pub fn read(fd: usize, buf: *mut u8, len: usize) -> isize {
        syscall3(Syscall::Read, fd, buf as usize, len)
    }

    pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
        syscall3(Syscall::Write, fd, buf as usize, len)
    }

    pub fn kill(pid: usize) -> isize {
        syscall1(Syscall::Kill, pid)
    }

    pub fn exec(path: *const u8, argv: *const *const u8) -> isize {
        syscall2(Syscall::Exec, path as usize, argv as usize)
    }

    pub fn fstat(fd: usize, stat: *mut Stat) -> isize {
        syscall2(Syscall::Fstat, fd, stat as usize)
    }

    pub fn chdir(path: *const u8) -> isize {
        syscall1(Syscall::Chdir, path as usize)
    }

    pub fn dup(fd: usize) -> isize {
        syscall1(Syscall::Dup, fd)
    }

    pub fn getpid() -> isize {
        syscall0(Syscall::Getpid)
    }

    pub fn sbrk(n: usize) -> isize {
        syscall1(Syscall::Sbrk, n)
    }

    pub fn sleep(ticks: usize) -> isize {
        syscall1(Syscall::Sleep, ticks)
    }

    pub fn uptime() -> isize {
        syscall0(Syscall::Uptime)
    }

    pub fn open(path: *const u8, flags: usize) -> isize {
        syscall2(Syscall::Open, path as usize, flags)
    }

    pub fn close(fd: usize) -> isize {
        syscall1(Syscall::Close, fd)
    }

    pub fn lseek(fd: usize, offset: isize, whence: i32) -> isize {
        syscall3(Syscall::Lseek, fd, offset as usize, whence as usize)
    }

    pub fn mknod(path: *const u8, major: usize, minor: usize) -> isize {
        syscall3(Syscall::Mknod, path as usize, major, minor)
    }

    pub fn unlink(path: *const u8) -> isize {
        syscall1(Syscall::Unlink, path as usize)
    }

    pub fn link(old: *const u8, new: *const u8) -> isize {
        syscall2(Syscall::Link, old as usize, new as usize)
    }

    pub fn mkdir(path: *const u8) -> isize {
        syscall1(Syscall::Mkdir, path as usize)
    }

    pub fn poweroff(code: u32) -> ! {
        syscall1(Syscall::Poweroff, code as usize);
        unreachable!();
    }

    pub fn ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
        syscall3(Syscall::Ioctl, fd, cmd, arg)
    }

    pub fn socket(port: u16) -> isize {
        syscall1(Syscall::Socket, port as usize)
    }

    pub fn send(
        fd: usize,
        buf: *const u8,
        len: usize,
        dest_ip: *const u8,
        dest_port: u16,
    ) -> isize {
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

    pub fn random(buf: *mut u8, len: usize) -> isize {
        syscall2(Syscall::Random, buf as usize, len)
    }

    pub fn dup2(oldfd: usize, newfd: usize) -> isize {
        syscall2(Syscall::Dup2, oldfd, newfd)
    }

    pub fn getppid() -> isize {
        syscall0(Syscall::Getppid)
    }

    pub fn setuid(uid: usize) -> isize {
        syscall1(Syscall::Setuid, uid)
    }

    pub fn setgid(gid: usize) -> isize {
        syscall1(Syscall::Setgid, gid)
    }

    pub fn setgroups(size: usize, list: *const u32) -> isize {
        syscall2(Syscall::Setgroups, size, list as usize)
    }

    pub fn getgroups(size: usize, list: *mut u32) -> isize {
        syscall2(Syscall::Getgroups, size, list as usize)
    }

    pub fn initgroups(user: *const u8, group: u32) -> isize {
        syscall2(Syscall::Initgroups, user as usize, group as usize)
    }

    pub fn sigaction(sig: usize, act: *const u8, oldact: *mut u8) -> isize {
        syscall3(Syscall::Sigaction, sig, act as usize, oldact as usize)
    }

    pub fn sigprocmask(how: i32, set: *const u32, oldset: *mut u32) -> isize {
        syscall3(Syscall::Sigprocmask, how as usize, set as usize, oldset as usize)
    }

    pub fn sigpending(set: *mut u32) -> isize {
        syscall1(Syscall::Sigpending, set as usize)
    }

    pub fn sigsuspend(mask: *const u32) -> isize {
        syscall1(Syscall::Sigsuspend, mask as usize)
    }

    pub fn sigreturn(ctx: *const u8) -> isize {
        syscall1(Syscall::Sigreturn, ctx as usize)
    }

    pub fn killpg(pgrp: usize, sig: usize) -> isize {
        syscall2(Syscall::Killpg, pgrp, sig)
    }

    pub fn getenv(name: *const u8, buf: *mut u8, len: usize) -> isize {
        syscall3(Syscall::Getenv, name as usize, buf as usize, len)
    }

    pub fn setenv(name: *const u8, value: *const u8, overwrite: isize) -> isize {
        syscall3(Syscall::Setenv, name as usize, value as usize, overwrite as usize)
    }

    pub fn unsetenv(name: *const u8) -> isize {
        syscall1(Syscall::Unsetenv, name as usize)
    }

    pub fn clearenv() -> isize {
        syscall0(Syscall::Clearenv)
    }

    pub fn getpagesize() -> isize {
        syscall0(Syscall::Getpagesize)
    }

    pub fn tcp_socket() -> isize {
        syscall0(Syscall::TcpSocket)
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

    pub fn fcntl(fd: usize, cmd: isize, arg: usize) -> isize {
        syscall3(Syscall::Fcntl, fd, cmd as usize, arg)
    }

    pub fn poll(fds: *mut kernel::abi::PollFd, nfds: usize, timeout: isize) -> isize {
        syscall3(Syscall::Poll, fds as usize, nfds, timeout as usize)
    }

    pub fn epoll_create1(flags: usize) -> isize {
        syscall1(Syscall::EpollCreate1, flags)
    }

    pub fn epoll_ctl(epfd: usize, op: usize, fd: usize, event: *const kernel::abi::EpollEvent) -> isize {
        syscall4(Syscall::EpollCtl, epfd, op, fd, event as usize)
    }

    pub fn epoll_wait(epfd: usize, events: *mut kernel::abi::EpollEvent, max_events: usize, timeout: isize) -> isize {
        syscall4(Syscall::EpollWait, epfd, events as usize, max_events, timeout as usize)
    }

    pub fn clone(flags: usize, stack: usize) -> isize {
        syscall2(Syscall::Clone, flags, stack)
    }

    pub fn clone_tls(flags: usize, stack: usize, ptid: usize, tls: usize) -> isize {
        syscall4(Syscall::Clone, flags, stack, ptid, tls)
    }

    pub fn gettid() -> isize {
        syscall0(Syscall::Gettid)
    }

    pub fn exit_group(code: usize) -> ! {
        syscall1(Syscall::ExitGroup, code);
        unreachable!();
    }

    pub fn getpgid(pid: usize) -> isize {
        syscall1(Syscall::Getpgid, pid)
    }

    pub fn isatty(fd: usize) -> isize {
        syscall1(Syscall::Isatty, fd)
    }

    pub fn tcgetattr(fd: usize, addr: usize) -> isize {
        syscall2(Syscall::Tcgetattr, fd, addr)
    }

    pub fn tcsetattr(fd: usize, addr: usize, opt: usize) -> isize {
        syscall3(Syscall::Tcsetattr, fd, addr, opt)
    }

    pub fn mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: isize, offset: usize) -> isize {
        syscall6(Syscall::Mmap, addr, length, prot, flags, fd as usize, offset)
    }

    pub fn munmap(addr: usize, length: usize) -> isize {
        syscall2(Syscall::Munmap, addr, length)
    }

    pub fn mprotect(addr: usize, length: usize, prot: usize) -> isize {
        syscall3(Syscall::Mprotect, addr, length, prot)
    }

    pub fn time(addr: usize) -> isize {
        syscall1(Syscall::Time, addr)
    }

    pub fn nanosleep(req: usize, _rem: usize) -> isize {
        syscall2(Syscall::Nanosleep, req, _rem)
    }

    pub fn clock_gettime(clock_id: usize, ts_addr: usize) -> isize {
        syscall2(Syscall::ClockGetTime, clock_id, ts_addr)
    }

    pub fn clock_getres(clock_id: usize, ts_addr: usize) -> isize {
        syscall2(Syscall::ClockGetRes, clock_id, ts_addr)
    }

    pub fn clock_settime(clock_id: usize, ts_addr: usize) -> isize {
        syscall2(Syscall::ClockSetTime, clock_id, ts_addr)
    }

    pub fn readv(fd: usize, iov: usize, iovcnt: usize) -> isize {
        syscall3(Syscall::Readv, fd, iov, iovcnt)
    }

    pub fn writev(fd: usize, iov: usize, iovcnt: usize) -> isize {
        syscall3(Syscall::Writev, fd, iov, iovcnt)
    }

    pub fn pread(fd: usize, buf: usize, n: usize, offset: usize) -> isize {
        syscall4(Syscall::Pread, fd, buf, n, offset)
    }

    pub fn pwrite(fd: usize, buf: usize, n: usize, offset: usize) -> isize {
        syscall4(Syscall::Pwrite, fd, buf, n, offset)
    }

    pub fn eventfd2(initval: u32, flags: u32) -> isize {
        let ret: isize;
        unsafe {
            asm!(
                "ecall",
                in("a7") Syscall::EventFd2 as usize,
                inlateout("a0") initval as isize => ret,
                in("a1") flags as usize,
            );
        }
        ret
    }

    pub fn memfd_create(flags: usize) -> isize {
        syscall2(Syscall::MemFdCreate, 0, flags)
    }

    pub fn pidfd_open(pid: usize, flags: usize) -> isize {
        syscall2(Syscall::PidFdOpen, pid, flags)
    }

    pub fn splice(fd_in: usize, off_in: *const i64, fd_out: usize, off_out: *const i64, len: usize, flags: u32) -> isize {
        syscall6(Syscall::Splice, fd_in, off_in as usize, fd_out, off_out as usize, len, flags as usize)
    }

    pub fn tee(fd_in: usize, fd_out: usize, len: usize, flags: u32) -> isize {
        syscall4(Syscall::Tee, fd_in, fd_out, len, flags as usize)
    }

    pub fn vmsplice(fd: usize, iov: usize, nr_segs: usize, flags: u32) -> isize {
        syscall4(Syscall::Vmsplice, fd, iov, nr_segs, flags as usize)
    }

    pub fn getrandom(buf: *mut u8, len: usize, flags: u32) -> isize {
        syscall3(Syscall::GetRandom, buf as usize, len, flags as usize)
    }

    pub fn close_range(first: usize, last: usize, flags: u32) -> isize {
        syscall3(Syscall::CloseRange, first, last, flags as usize)
    }

    pub fn inotify_init1(flags: u32) -> isize {
        syscall1(Syscall::InotifyInit1, flags as usize)
    }

    pub fn inotify_add_watch(fd: usize, pathname: *const u8, mask: u32) -> isize {
        syscall3(Syscall::InotifyAddWatch, fd, pathname as usize, mask as usize)
    }

    pub fn inotify_rm_watch(fd: usize, wd: i32) -> isize {
        syscall2(Syscall::InotifyRmWatch, fd, wd as usize)
    }

    pub fn signalfd4(fd: usize, mask: *const u32, sizemask: usize, flags: u32) -> isize {
        syscall4(Syscall::Signalfd4, fd, mask as usize, sizemask, flags as usize)
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

    pub fn nsopen(pid: usize, nstype: u32) -> isize {
        syscall2(Syscall::NsOpen, pid, nstype as usize)
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

    pub fn pivot_root(new_root: *const u8, put_old: *const u8) -> isize {
        syscall2(Syscall::PivotRoot, new_root as usize, put_old as usize)
    }
}

use kernel::abi::{MAXPATH, Stat, SysError};

/// A file descriptor returned by or passed to syscalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fd(usize);

impl Fd {
    pub const STDIN: Fd = Fd(0);
    pub const STDOUT: Fd = Fd(1);
    pub const STDERR: Fd = Fd(2);

    /// Returns the raw file descriptor number.
    pub fn as_raw(&self) -> usize {
        self.0
    }

    /// Creates a new Fd from a raw file descriptor number.
    pub fn from_raw(raw: usize) -> Self {
        Fd(raw)
    }
}

impl core::fmt::Display for Fd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated path suitable for passing to syscalls.
///
/// Guarantees that the inner string is shorter than `MAXPATH` and contains no
/// embedded null bytes, so it can be safely null-terminated on the stack.
#[derive(Debug, Clone, Copy)]
struct Path<'a>(&'a str);

impl<'a> Path<'a> {
    fn new(s: &'a str) -> Result<Self, SysError> {
        if s.len() >= MAXPATH || s.bytes().any(|b| b == 0) {
            return Err(SysError::NameTooLong);
        }
        Ok(Self(s))
    }

    /// Creates a null-terminated C-string buffer on the stack.
    fn as_cpath(&self) -> [u8; MAXPATH] {
        let mut buf = [0u8; MAXPATH];
        buf[..self.0.len()].copy_from_slice(self.0.as_bytes());
        buf
    }
}

/// Converts a raw signed syscall return into `Result`, treating negative values as error codes.
#[inline(always)]
fn check(ret: isize) -> Result<usize, SysError> {
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(SysError::from_code((-ret) as u16))
    }
}

/// Converts a raw syscall return into `Result<(), SysError>`.
#[inline(always)]
fn check_unit(ret: isize) -> Result<(), SysError> {
    check(ret).map(|_| ())
}

/// Validates a path string and creates a C-compatible path buffer.
fn validate_path(path: &str) -> Result<[u8; MAXPATH], SysError> {
    Ok(Path::new(path)?.as_cpath())
}

pub fn fork() -> Result<usize, SysError> {
    check(raw::fork())
}

pub fn exit(code: usize) -> ! {
    raw::exit(code)
}

pub fn exit_with_msg(msg: &str) -> ! {
    eprintln!("{}", msg);
    exit(1);
}

pub fn wait(status: &mut usize) -> Result<usize, SysError> {
    check(raw::wait(status as *mut usize))
}

pub fn pipe() -> Result<(Fd, Fd), SysError> {
    let mut fds = [0usize; 2];
    check_unit(raw::pipe(fds.as_mut_ptr()))?;
    Ok((Fd(fds[0]), Fd(fds[1])))
}

pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize, SysError> {
    check(raw::read(fd.as_raw(), buf.as_mut_ptr(), buf.len()))
}

pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, SysError> {
    check(raw::write(fd.as_raw(), buf.as_ptr(), buf.len()))
}

pub fn kill(pid: usize) -> Result<(), SysError> {
    check_unit(raw::kill(pid))
}

/// Replaces the current process image with the program at `path`.
///
/// `argv` contains the argument strings. This function packs them into a contiguous
/// stack buffer with null terminators and builds the pointer array expected by the kernel.
///
/// Returns `SysError` because if `exec` returns at all, it failed.
pub fn exec(path: &str, argv: &[&str]) -> SysError {
    let cpath = match validate_path(path) {
        Ok(cpath) => cpath,
        Err(e) => return e,
    };

    const MAX_ARGV: usize = 16;
    const BUF_SIZE: usize = 512;

    let mut buf = [0u8; BUF_SIZE];
    let mut ptrs: [*const u8; MAX_ARGV + 1] = [core::ptr::null(); MAX_ARGV + 1];
    let mut offset = 0;

    for (i, arg) in argv.iter().enumerate().take(MAX_ARGV) {
        ptrs[i] = buf[offset..].as_ptr();
        buf[offset..offset + arg.len()].copy_from_slice(arg.as_bytes());
        // buf is zeroed, so the byte after the arg is already a null terminator
        offset += arg.len() + 1;
    }
    // ptrs is already null-terminated (initialized to null)

    let ret = raw::exec(cpath.as_ptr(), ptrs.as_ptr());
    // exec only returns on failure
    SysError::from_code((-ret) as u16)
}

pub fn fstat(fd: Fd, stat: &mut Stat) -> Result<(), SysError> {
    check_unit(raw::fstat(fd.as_raw(), stat as *mut Stat))
}

pub fn chdir(path: &str) -> Result<(), SysError> {
    let cpath = validate_path(path)?;
    check_unit(raw::chdir(cpath.as_ptr()))
}

pub fn dup(fd: Fd) -> Result<Fd, SysError> {
    check(raw::dup(fd.as_raw())).map(Fd)
}

pub fn getpid() -> usize {
    raw::getpid() as usize
}

pub fn sbrk(n: isize) -> Result<usize, SysError> {
    check(raw::sbrk(n as usize))
}

pub fn sleep(ticks: usize) -> Result<(), SysError> {
    check_unit(raw::sleep(ticks))
}

pub fn uptime() -> usize {
    raw::uptime() as usize
}

pub fn open(path: &str, flags: usize) -> Result<Fd, SysError> {
    let cpath = validate_path(path)?;
    check(raw::open(cpath.as_ptr(), flags)).map(Fd)
}

pub fn close(fd: Fd) -> Result<(), SysError> {
    check_unit(raw::close(fd.as_raw()))
}

pub fn lseek(fd: Fd, offset: i64, whence: i32) -> Result<i64, SysError> {
    check(raw::lseek(fd.as_raw(), offset as isize, whence)).map(|v| v as i64)
}

pub fn mknod(path: &str, major: usize, minor: usize) -> Result<(), SysError> {
    let cpath = validate_path(path)?;
    check_unit(raw::mknod(cpath.as_ptr(), major, minor))
}

pub fn unlink(path: &str) -> Result<(), SysError> {
    let cpath = validate_path(path)?;
    check_unit(raw::unlink(cpath.as_ptr()))
}

pub fn link(old: &str, new: &str) -> Result<(), SysError> {
    let cold = validate_path(old)?;
    let cnew = validate_path(new)?;
    check_unit(raw::link(cold.as_ptr(), cnew.as_ptr()))
}

pub fn mkdir(path: &str) -> Result<(), SysError> {
    let cpath = validate_path(path)?;
    check_unit(raw::mkdir(cpath.as_ptr()))
}

pub fn poweroff(code: u32) -> ! {
    raw::poweroff(code)
}

pub fn ioctl(fd: Fd, cmd: usize, arg: usize) -> Result<usize, SysError> {
    check(raw::ioctl(fd.as_raw(), cmd, arg))
}

pub fn socket(port: u16) -> Result<Fd, SysError> {
    check(raw::socket(port)).map(Fd)
}

pub fn send(fd: Fd, buf: &[u8], dest_ip: &[u8; 4], dest_port: u16) -> Result<usize, SysError> {
    check(raw::send(
        fd.as_raw(),
        buf.as_ptr(),
        buf.len(),
        dest_ip.as_ptr(),
        dest_port,
    ))
}

pub fn receive(
    fd: Fd,
    buf: &mut [u8],
    src_ip: &mut [u8; 4],
    src_port: &mut u16,
) -> Result<usize, SysError> {
    check(raw::receive(
        fd.as_raw(),
        buf.as_mut_ptr(),
        buf.len(),
        src_ip.as_mut_ptr(),
        src_port as *mut u16,
    ))
}

pub fn random(buf: &mut [u8]) -> Result<(), SysError> {
    check_unit(raw::random(buf.as_mut_ptr(), buf.len()))
}

pub fn dup2(oldfd: Fd, newfd: Fd) -> Result<Fd, SysError> {
    check(raw::dup2(oldfd.as_raw(), newfd.as_raw())).map(Fd)
}

pub fn getppid() -> Result<usize, SysError> {
    check(raw::getppid())
}

pub fn setuid(uid: u32) -> Result<(), SysError> {
    check_unit(raw::setuid(uid as usize))
}

pub fn setgid(gid: u32) -> Result<(), SysError> {
    check_unit(raw::setgid(gid as usize))
}

pub fn getpgid(pid: usize) -> Result<usize, SysError> {
    check(raw::getpgid(pid))
}

pub fn isatty(fd: Fd) -> Result<bool, SysError> {
    check(raw::isatty(fd.as_raw())).map(|v| v == 1)
}

pub fn tcgetattr(_fd: Fd, _attr: &mut [u8]) -> Result<(), SysError> {
    check_unit(raw::tcgetattr(_fd.as_raw(), _attr.as_mut_ptr() as usize))
}

pub fn tcsetattr(_fd: Fd, _attr: &[u8], _opt: usize) -> Result<(), SysError> {
    check_unit(raw::tcsetattr(_fd.as_raw(), _attr.as_ptr() as usize, _opt))
}

pub fn mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: isize, offset: usize) -> Result<usize, SysError> {
    check(raw::mmap(addr, length, prot, flags, fd, offset))
}

pub fn munmap(addr: usize, length: usize) -> Result<(), SysError> {
    check_unit(raw::munmap(addr, length))
}

pub fn mprotect(addr: usize, length: usize, prot: usize) -> Result<(), SysError> {
    check_unit(raw::mprotect(addr, length, prot))
}

pub fn time() -> Result<usize, SysError> {
    check(raw::time(0))
}

pub fn nanosleep(sec: u64, nsec: u64) -> Result<(), SysError> {
    let req = [sec, nsec];
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&req[0].to_le_bytes());
    buf[8..].copy_from_slice(&req[1].to_le_bytes());
    check_unit(raw::nanosleep(&buf as *const _ as usize, 0))
}

pub fn clock_gettime() -> Result<(u64, u64), SysError> {
    let mut ts = [0u8; 16];
    check(raw::clock_gettime(0, ts.as_mut_ptr() as usize))?;
    let sec = u64::from_le_bytes([ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7]]);
    let nsec = u64::from_le_bytes([ts[8], ts[9], ts[10], ts[11], ts[12], ts[13], ts[14], ts[15]]);
    Ok((sec, nsec))
}

pub fn clock_getres() -> Result<(u64, u64), SysError> {
    let mut ts = [0u8; 16];
    check(raw::clock_getres(0, ts.as_mut_ptr() as usize))?;
    let sec = u64::from_le_bytes([ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7]]);
    let nsec = u64::from_le_bytes([ts[8], ts[9], ts[10], ts[11], ts[12], ts[13], ts[14], ts[15]]);
    Ok((sec, nsec))
}

pub fn clock_settime(_sec: u64, _nsec: u64) -> Result<(), SysError> {
    check_unit(raw::clock_settime(0, 0))
}

#[repr(C)]
pub struct Iovec {
    pub iov_base: *mut u8,
    pub iov_len: usize,
}

pub fn readv(fd: Fd, iovs: &mut [Iovec]) -> Result<usize, SysError> {
    check(raw::readv(fd.as_raw(), iovs.as_mut_ptr() as usize, iovs.len()))
}

pub fn writev(fd: Fd, iovs: &[Iovec]) -> Result<usize, SysError> {
    check(raw::writev(fd.as_raw(), iovs.as_ptr() as usize, iovs.len()))
}

pub fn pread(fd: Fd, buf: &mut [u8], offset: usize) -> Result<usize, SysError> {
    check(raw::pread(fd.as_raw(), buf.as_mut_ptr() as usize, buf.len(), offset))
}

pub fn pwrite(fd: Fd, buf: &[u8], offset: usize) -> Result<usize, SysError> {
    check(raw::pwrite(fd.as_raw(), buf.as_ptr() as usize, buf.len(), offset))
}

pub fn mkfifo(path: &str) -> Result<(), SysError> {
    let cpath = validate_path(path)?;
    check_unit(raw::mkfifo(cpath.as_ptr()))
}

pub fn pipe2(flags: usize) -> Result<(Fd, Fd), SysError> {
    let mut fds = [0usize; 2];
    check_unit(raw::pipe2(fds.as_mut_ptr(), flags))?;
    Ok((Fd(fds[0]), Fd(fds[1])))
}

pub fn setgroups(groups: &[u32]) -> Result<(), SysError> {
    check_unit(raw::setgroups(groups.len(), groups.as_ptr()))
}

pub fn getgroups(groups: &mut [u32]) -> Result<usize, SysError> {
    check(raw::getgroups(groups.len(), groups.as_mut_ptr()))
}

pub fn initgroups(user: &str, group: u32) -> Result<(), SysError> {
    if user.len() >= 256 || user.bytes().any(|b| b == 0) {
        return Err(SysError::NameTooLong);
    }
    let mut buf = [0u8; 256];
    buf[..user.len()].copy_from_slice(user.as_bytes());
    check_unit(raw::initgroups(buf.as_ptr(), group))
}

pub fn sigaction(sig: usize, act: Option<&kernel::abi::SigAction>,
                 oldact: Option<&mut kernel::abi::SigAction>) -> Result<(), SysError> {
    let act_ptr = act.map(|a| a as *const _ as *const u8).unwrap_or(core::ptr::null());
    let oldact_ptr = oldact.map(|a| a as *mut _ as *mut u8).unwrap_or(core::ptr::null_mut());
    check_unit(raw::sigaction(sig, act_ptr, oldact_ptr))
}

pub fn sigprocmask(how: i32, set: Option<u32>) -> Result<u32, SysError> {
    let mut oldset: u32 = 0;
    let set_val = set.unwrap_or(0);
    check(raw::sigprocmask(how, &set_val as *const u32, &mut oldset as *mut u32))?;
    Ok(oldset)
}

pub fn sigpending() -> Result<u32, SysError> {
    let mut set: u32 = 0;
    check_unit(raw::sigpending(&mut set as *mut u32))?;
    Ok(set)
}

pub fn sigsuspend(mask: u32) -> Result<(), SysError> {
    check_unit(raw::sigsuspend(&mask as *const u32))
}

pub fn sigreturn(ctx: *const u8) -> ! {
    raw::sigreturn(ctx);
    unreachable!()
}

pub fn killpg(pgrp: usize, sig: usize) -> Result<(), SysError> {
    check_unit(raw::killpg(pgrp, sig))
}

fn validate_env_name(name: &str) -> Result<[u8; 256], SysError> {
    if name.is_empty() || name.len() >= 256 || name.bytes().any(|b| b == 0) {
        return Err(SysError::InvalidArgument);
    }
    let mut buf = [0u8; 256];
    buf[..name.len()].copy_from_slice(name.as_bytes());
    Ok(buf)
}

pub fn getenv(name: &str, buf: &mut [u8]) -> Result<usize, SysError> {
    let cname = validate_env_name(name)?;
    check(raw::getenv(cname.as_ptr(), buf.as_mut_ptr(), buf.len()))
}

pub fn setenv(name: &str, value: &str, overwrite: bool) -> Result<(), SysError> {
    let cname = validate_env_name(name)?;
    if value.len() >= 256 || value.bytes().any(|b| b == 0) {
        return Err(SysError::InvalidArgument);
    }
    let mut cvalue = [0u8; 256];
    cvalue[..value.len()].copy_from_slice(value.as_bytes());
    check_unit(raw::setenv(cname.as_ptr(), cvalue.as_ptr(), overwrite as isize))
}

pub fn unsetenv(name: &str) -> Result<(), SysError> {
    let cname = validate_env_name(name)?;
    check_unit(raw::unsetenv(cname.as_ptr()))
}

pub fn clearenv() -> Result<(), SysError> {
    check_unit(raw::clearenv())
}

pub fn getpagesize() -> usize {
    raw::getpagesize() as usize
}

pub fn tcp_socket() -> Result<Fd, SysError> {
    check(raw::tcp_socket()).map(Fd)
}

pub fn tcp_bind(fd: Fd, port: u16) -> Result<(), SysError> {
    check_unit(raw::tcp_bind(fd.as_raw(), port))
}

pub fn tcp_listen(fd: Fd) -> Result<(), SysError> {
    check_unit(raw::tcp_listen(fd.as_raw()))
}

pub fn tcp_accept(fd: Fd) -> Result<Fd, SysError> {
    check(raw::tcp_accept(fd.as_raw())).map(Fd)
}

pub fn tcp_connect(fd: Fd, dest_ip: &[u8; 4], dest_port: u16) -> Result<(), SysError> {
    check_unit(raw::tcp_connect(fd.as_raw(), dest_ip.as_ptr(), dest_port))
}

pub fn tcp_send(fd: Fd, buf: &[u8]) -> Result<usize, SysError> {
    check(raw::tcp_send(fd.as_raw(), buf.as_ptr(), buf.len()))
}

pub fn tcp_recv(fd: Fd, buf: &mut [u8]) -> Result<usize, SysError> {
    check(raw::tcp_recv(fd.as_raw(), buf.as_mut_ptr(), buf.len()))
}

pub fn fcntl(fd: Fd, cmd: isize, arg: usize) -> Result<usize, SysError> {
    check(raw::fcntl(fd.as_raw(), cmd, arg))
}

pub fn set_nonblocking(fd: Fd) -> Result<(), SysError> {
    check_unit(raw::fcntl(fd.as_raw(), 4, 0x800)) // F_SETFL=4, O_NONBLOCK=0x800
}

pub fn poll(fds: &mut [kernel::abi::PollFd], timeout: isize) -> Result<usize, SysError> {
    check(raw::poll(fds.as_mut_ptr(), fds.len(), timeout))
}

pub fn epoll_create1(flags: usize) -> Result<Fd, SysError> {
    check(raw::epoll_create1(flags)).map(Fd)
}

pub fn epoll_ctl(epfd: Fd, op: usize, fd: Fd, event: Option<&kernel::abi::EpollEvent>) -> Result<(), SysError> {
    let ptr = match event {
        Some(e) => e as *const _,
        None => core::ptr::null(),
    };
    check_unit(raw::epoll_ctl(epfd.as_raw(), op, fd.as_raw(), ptr))
}

pub fn epoll_wait(epfd: Fd, events: &mut [kernel::abi::EpollEvent], timeout: isize) -> Result<usize, SysError> {
    check(raw::epoll_wait(epfd.as_raw(), events.as_mut_ptr(), events.len(), timeout))
}

pub fn splice(fd_in: Fd, off_in: *const i64, fd_out: Fd, off_out: *const i64, len: usize, flags: u32) -> Result<usize, SysError> {
    check(raw::splice(fd_in.as_raw(), off_in, fd_out.as_raw(), off_out, len, flags))
}

pub fn tee(fd_in: Fd, fd_out: Fd, len: usize, flags: u32) -> Result<usize, SysError> {
    check(raw::tee(fd_in.as_raw(), fd_out.as_raw(), len, flags))
}

pub fn clone(flags: usize, stack: usize) -> Result<usize, SysError> {
    check(raw::clone(flags, stack))
}

/// Like `clone()` but also passes a TLS pointer (CLONE_SETTLS).
/// `ptid` is the parent TID address (unused in xv8).
/// `tls` is the thread-local storage pointer for `tp` register.
pub fn clone_with_tls(flags: usize, stack: usize, ptid: usize, tls: usize) -> Result<usize, SysError> {
    check(raw::clone_tls(flags, stack, ptid, tls))
}

pub fn gettid() -> usize {
    raw::gettid() as usize
}

pub fn exit_group(code: usize) -> ! {
    raw::exit_group(code)
}

pub fn inotify_init1(flags: u32) -> Result<Fd, SysError> {
    check(raw::inotify_init1(flags)).map(Fd)
}

pub fn inotify_add_watch(fd: Fd, path: &str, mask: u32) -> Result<i32, SysError> {
    let cpath = validate_path(path)?;
    let ret = raw::inotify_add_watch(fd.as_raw(), cpath.as_ptr(), mask);
    if ret >= 0 { Ok(ret as i32) } else { Err(SysError::from_code((-ret) as u16)) }
}

pub fn inotify_rm_watch(fd: Fd, wd: i32) -> Result<(), SysError> {
    check_unit(raw::inotify_rm_watch(fd.as_raw(), wd))
}

pub fn signalfd4(mask: u32, flags: u32) -> Result<Fd, SysError> {
    check(raw::signalfd4(0, &mask as *const u32, core::mem::size_of::<u32>(), flags)).map(Fd)
}

pub fn timerfd_create(clockid: i32, flags: u32) -> Result<Fd, SysError> {
    check(raw::timerfd_create(clockid, flags)).map(Fd)
}

pub fn timerfd_settime(fd: Fd, flags: u32, new_val: usize) -> Result<(), SysError> {
    check_unit(raw::timerfd_settime(fd.as_raw(), flags, new_val, 0))
}

pub fn timerfd_gettime(fd: Fd, curr_val: usize) -> Result<(), SysError> {
    check_unit(raw::timerfd_gettime(fd.as_raw(), curr_val))
}

pub fn setns(fd: Fd, nstype: u32) -> Result<(), SysError> {
    check_unit(raw::setns(fd.as_raw(), nstype))
}

pub fn nsopen(pid: usize, nstype: u32) -> Result<Fd, SysError> {
    check(raw::nsopen(pid, nstype)).map(|fd| Fd(fd as _))
}

pub fn unshare(flags: usize) -> Result<(), SysError> {
    check_unit(raw::unshare(flags))
}

pub fn sethostname(name: &[u8]) -> Result<(), SysError> {
    check_unit(raw::sethostname(name.as_ptr(), name.len()))
}

pub fn gethostname(buf: &mut [u8]) -> Result<usize, SysError> {
    let ret = raw::gethostname(buf.as_mut_ptr(), buf.len());
    if ret < 0 {
        Err(SysError::from_code((-ret) as u16))
    } else {
        Ok(ret as usize)
    }
}

pub fn cap_get_pid(_pid: u32) -> Result<u64, SysError> {
    let mut data = [0usize; 3];
    check_unit(raw::capget(core::ptr::null(), data.as_mut_ptr()))?;
    Ok(data[0] as u64)
}

pub fn cap_set(effective: u64, permitted: u64, inheritable: u64) -> Result<(), SysError> {
    let data = [effective as usize, permitted as usize, inheritable as usize];
    check_unit(raw::capset(core::ptr::null(), data.as_ptr()))
}

pub fn seccomp_filter(filter: &[u8]) -> Result<(), SysError> {
    let ptr = filter.as_ptr() as usize;
    // sock_fprog: u16 len + 6 bytes padding (riscv64) + u64 ptr
    let mut buf = [0u8; 16];
    buf[0..2].copy_from_slice(&(filter.len() as u16).to_ne_bytes());
    buf[8..16].copy_from_slice(&ptr.to_ne_bytes());
    check_unit(raw::seccomp(2, 0, buf.as_ptr()))
}

pub fn overlay_mount(mount_point: &str, upper: &str, lower: &str) -> Result<(), SysError> {
    // Null-terminate for kernel fetch_string (Rust &str may lack \0 in .rodata)
    fn null_terminate(s: &str, buf: &mut [u8; 256]) -> *const u8 {
        let len = s.len().min(255);
        buf[..len].copy_from_slice(s.as_bytes());
        buf[len] = 0;
        buf.as_ptr()
    }
    let mut mp_buf = [0u8; 256];
    let mut up_buf = [0u8; 256];
    let mut lo_buf = [0u8; 256];
    check_unit(raw::overlay_mount(
        null_terminate(mount_point, &mut mp_buf),
        null_terminate(upper, &mut up_buf),
        null_terminate(lower, &mut lo_buf),
    ))
}

pub fn overlay_umount(mount_point: &str) -> Result<(), SysError> {
    let mut buf = [0u8; 256];
    let len = mount_point.len().min(255);
    buf[..len].copy_from_slice(mount_point.as_bytes());
    buf[len] = 0;
    check_unit(raw::overlay_umount(buf.as_ptr()))
}

pub fn pivot_root(new_root: &str, put_old: &str) -> Result<(), SysError> {
    let mut nr_buf = [0u8; 256];
    let mut po_buf = [0u8; 256];
    let nr_len = new_root.len().min(255);
    let po_len = put_old.len().min(255);
    nr_buf[..nr_len].copy_from_slice(new_root.as_bytes());
    nr_buf[nr_len] = 0;
    po_buf[..po_len].copy_from_slice(put_old.as_bytes());
    po_buf[po_len] = 0;
    check_unit(raw::pivot_root(nr_buf.as_ptr(), po_buf.as_ptr()))
}

pub fn seccomp_kill() -> Result<(), SysError> {
    // SECCOMP_SET_MODE_FILTER(2) with a filter that always kills
    let filter: [u8; 8] = [
        0x06, 0x00, 0x00, 0x00,  // ret
        0x00, 0x00, 0x00, 0x80,  // SECCOMP_RET_KILL_PROCESS
    ];
    seccomp_filter(&filter)
}
