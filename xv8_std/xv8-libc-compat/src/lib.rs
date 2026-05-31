#![no_std]

// On non-riscv64, delegate to the real libc crate
#[cfg(not(target_arch = "riscv64"))]
pub use real_libc::*;

// On riscv64, provide our own minimal libc implementation
#[cfg(target_arch = "riscv64")]
mod riscv64_impl {
    #![allow(non_camel_case_types, non_upper_case_globals)]

    // ============================================================
    // Type aliases
    // ============================================================

    pub type c_int = i32;
    pub type c_char = i8;
    pub type c_uint = u32;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_long = i64;
    pub type c_ulong = u64;
    pub type c_schar = i8;
    pub type c_uchar = u8;
    pub type c_float = f32;
    pub type c_double = f64;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
    pub type size_t = usize;
    pub type ssize_t = isize;
    pub type off_t = i64;
    pub use core::ffi::c_void;

    pub type pid_t = c_int;
    pub type uid_t = u32;
    pub type gid_t = u32;
    pub type mode_t = u32;
    pub type tcflag_t = c_uint;
    pub type cc_t = c_uchar;
    pub type speed_t = c_uint;

    // ============================================================
    // Constants
    // ============================================================

    pub const SIGCONT: c_int = 18;
    pub const WNOHANG: c_int = 1;
    pub const WUNTRACED: c_int = 4;
    pub const STDIN_FILENO: c_int = 0;
    pub const STDOUT_FILENO: c_int = 1;
    pub const STDERR_FILENO: c_int = 2;
    pub const NCCS: usize = 32;
    pub const TCSAFLUSH: c_int = 2;

    // c_iflag
    pub const BRKINT: tcflag_t = 0x0002;
    pub const ICRNL: tcflag_t = 0x0100;
    pub const IXON: tcflag_t = 0x0400;
    pub const INPCK: tcflag_t = 0x0010;
    pub const ISTRIP: tcflag_t = 0x0020;

    // c_oflag
    pub const OPOST: tcflag_t = 0x0001;

    // c_cflag
    pub const CS8: tcflag_t = 0x0030;

    // c_lflag
    pub const ECHO: tcflag_t = 0x0008;
    pub const ICANON: tcflag_t = 0x0002;
    pub const ISIG: tcflag_t = 0x0001;
    pub const IEXTEN: tcflag_t = 0x8000;

    // c_cc indexes
    pub const VMIN: usize = 6;
    pub const VTIME: usize = 5;

    // ioctl
    pub const TIOCGWINSZ: c_ulong = 0x5413;

    // IPC
    pub const IPC_RMID: c_int = 0;
    pub const IPC_STAT: c_int = 2;

    // sysconf
    pub const _SC_GETGR_R_SIZE_MAX: c_int = 69;

    // ============================================================
    // Structs
    // ============================================================

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct utsname {
        pub sysname: [c_char; 65],
        pub nodename: [c_char; 65],
        pub release: [c_char; 65],
        pub version: [c_char; 65],
        pub machine: [c_char; 65],
        pub domainname: [c_char; 65],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct termios {
        pub c_iflag: tcflag_t,
        pub c_oflag: tcflag_t,
        pub c_cflag: tcflag_t,
        pub c_lflag: tcflag_t,
        pub c_line: cc_t,
        pub c_cc: [cc_t; NCCS],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct winsize {
        pub ws_row: u16,
        pub ws_col: u16,
        pub ws_xpixel: u16,
        pub ws_ypixel: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct passwd {
        pub pw_name: *mut c_char,
        pub pw_passwd: *mut c_char,
        pub pw_uid: uid_t,
        pub pw_gid: gid_t,
        pub pw_gecos: *mut c_char,
        pub pw_dir: *mut c_char,
        pub pw_shell: *mut c_char,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct group {
        pub gr_name: *mut c_char,
        pub gr_passwd: *mut c_char,
        pub gr_gid: gid_t,
        pub gr_mem: *mut *mut c_char,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ipc_perm {
        pub _key: c_int,
        pub uid: uid_t,
        pub gid: gid_t,
        pub cuid: uid_t,
        pub cgid: gid_t,
        pub mode: c_ushort,
        pub _pad1: c_ushort,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct shmid_ds {
        pub shm_perm: ipc_perm,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct semid_ds {
        pub sem_perm: ipc_perm,
    }

    // ============================================================
    // Raw syscall wrappers
    // ============================================================

    mod raw {
        #[repr(usize)]
        enum Syscall {
            Kill = 6,
            Wait = 3,
            Uname = 43,
            Mkfifo = 83,
            Setuid = 56,
            Tcgetattr = 60,
            Tcsetattr = 61,
            Ioctl = 23,
            Ttyioctl = 86,
            Setregid = 78,
            Sysconf = 92,
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

        pub fn raw_kill(pid: i32) -> isize {
            syscall1(Syscall::Kill, pid as usize)
        }

        pub fn raw_wait(status: *mut usize) -> isize {
            syscall1(Syscall::Wait, status as usize)
        }

        pub fn raw_uname(buf: *mut u8) -> isize {
            syscall1(Syscall::Uname, buf as usize)
        }

        pub fn raw_mkfifo(path: *const u8) -> isize {
            syscall1(Syscall::Mkfifo, path as usize)
        }

        pub fn raw_tcgetattr(fd: usize, buf: *mut u8) -> isize {
            syscall2(Syscall::Tcgetattr, fd, buf as usize)
        }

        pub fn raw_tcsetattr(fd: usize, buf: *const u8, opt: usize) -> isize {
            syscall3(Syscall::Tcsetattr, fd, buf as usize, opt)
        }

        pub fn raw_ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
            syscall3(Syscall::Ioctl, fd, cmd, arg)
        }

        pub fn raw_ttyioctl(fd: usize, cmd: usize, arg: usize) -> isize {
            syscall3(Syscall::Ttyioctl, fd, cmd, arg)
        }

        pub fn raw_setregid(rgid: usize, egid: usize) -> isize {
            syscall2(Syscall::Setregid, rgid, egid)
        }

        pub fn raw_setuid(uid: usize) -> isize {
            syscall1(Syscall::Setuid, uid)
        }

        pub fn raw_sysconf(name: usize) -> isize {
            syscall1(Syscall::Sysconf, name)
        }
    }

    // ============================================================
    // Helper
    // ============================================================

    #[inline]
    fn check(ret: isize) -> c_int {
        if ret >= 0 {
            ret as c_int
        } else {
            -1
        }
    }

    // ============================================================
    // Functions backed by real syscalls
    // ============================================================

    pub unsafe fn uname(buf: *mut utsname) -> c_int {
        check(raw::raw_uname(buf as *mut u8))
    }

    pub unsafe fn mkfifo(pathname: *const c_char, _mode: mode_t) -> c_int {
        check(raw::raw_mkfifo(pathname as *const u8))
    }

    pub unsafe fn kill(pid: pid_t, _sig: c_int) -> c_int {
        check(raw::raw_kill(pid))
    }

    pub unsafe fn waitpid(_pid: pid_t, status: *mut c_int, _options: c_int) -> pid_t {
        let ret = raw::raw_wait(status as *mut c_int as *mut usize);
        if ret >= 0 {
            ret as pid_t
        } else {
            -1
        }
    }

    #[allow(non_snake_case)]
    pub fn WIFSTOPPED(status: c_int) -> bool {
        (status & 0xff) == 0x7f
    }

    pub unsafe fn tcsetpgrp(fd: c_int, pgrp: pid_t) -> c_int {
        check(raw::raw_ttyioctl(fd as usize, 2, pgrp as usize))
    }

    pub unsafe fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int {
        check(raw::raw_tcgetattr(fd as usize, termios_p as *mut u8))
    }

    pub unsafe fn tcsetattr(fd: c_int, _optional_actions: c_int, termios_p: *const termios) -> c_int {
        check(raw::raw_tcsetattr(fd as usize, termios_p as *const u8, _optional_actions as usize))
    }

    pub unsafe fn ioctl<T>(fd: c_int, request: c_ulong, argp: *mut T) -> c_int {
        check(raw::raw_ioctl(fd as usize, request as usize, argp as usize))
    }

    pub unsafe fn setregid(rgid: gid_t, egid: gid_t) -> c_int {
        check(raw::raw_setregid(rgid as usize, egid as usize))
    }

    pub unsafe fn setuid(uid: uid_t) -> c_int {
        check(raw::raw_setuid(uid as usize))
    }

    pub unsafe fn sysconf(name: c_int) -> c_long {
        let ret = raw::raw_sysconf(name as usize);
        if ret >= 0 {
            ret as c_long
        } else {
            -1
        }
    }

    // ============================================================
    // Stub functions
    // ============================================================

    pub unsafe fn shmctl<T>(_shmid: c_int, _cmd: c_int, _buf: *mut T) -> c_int {
        -1
    }

    pub unsafe fn semctl<T>(_semid: c_int, _semnum: c_int, _cmd: c_int, _arg: *mut T) -> c_int {
        -1
    }

    pub unsafe fn getpwnam(_name: *const c_char) -> *mut passwd {
        core::ptr::null_mut()
    }

    pub unsafe fn getgrnam_r(
        _name: *const c_char,
        _grp: *mut group,
        _buf: *mut c_char,
        _buflen: size_t,
        _result: *mut *mut group,
    ) -> c_int {
        -1
    }
}

#[cfg(target_arch = "riscv64")]
pub use riscv64_impl::*;
