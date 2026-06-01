

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
