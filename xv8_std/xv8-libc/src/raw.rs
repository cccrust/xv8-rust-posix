

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Read = 5,
    Write = 16,
    Open = 15,
    Close = 21,
    Lseek = 28,
    Fstat = 8,
    Exit = 2,
    Getpid = 11,
    Chdir = 9,
    Sbrk = 12,
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

pub fn read(fd: usize, buf: *mut u8, len: usize) -> isize {
    syscall3(Syscall::Read, fd, buf as usize, len)
}

pub fn write(fd: usize, buf: *const u8, len: usize) -> isize {
    syscall3(Syscall::Write, fd, buf as usize, len)
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

pub fn sbrk(n: isize) -> isize {
    syscall1(Syscall::Sbrk, n as usize)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub nlink: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub _pad0: u32,
    pub size: u64,
    pub blksize: u32,
    pub blocks: u32,
    pub _pad1: u64,
}