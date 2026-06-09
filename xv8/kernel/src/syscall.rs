use core::fmt::Display;

use alloc::string::String;

use crate::file::File;
use crate::fs::FsError;
use crate::net::NetError;
use crate::param::NOFILE;
use crate::proc::{Proc, TrapFrame, current_proc, current_proc_and_data_mut};
use crate::poll::*;
use crate::sysfile::*;
use crate::sysnet::*;
use crate::sysproc::*;
use crate::vm::VA;

/// Syscall error codes using POSIX-standard numeric values.
///
/// Kernel encodes `-(error_code as isize)` in the return register (`a0`).
/// User space decodes negative values back into `SysError` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SysError {
    NotPermitted = 1,
    NoEntry = 2,
    NoProcess = 3,
    Interrupted = 4,
    IoError = 5,
    InvalidExecutable = 8,
    BadDescriptor = 9,
    NoChildren = 10,
    ResourceUnavailable = 11,
    OutOfMemory = 12,
    BadAddress = 14,
    AlreadyExists = 17,
    CrossDeviceLink = 18,
    NotDirectory = 20,
    IsDirectory = 21,
    InvalidArgument = 22,
    FileTableFull = 23,
    TooManyFiles = 24,
    NoSpace = 28,
    TooManyLinks = 31,
    BrokenPipe = 32,
    NameTooLong = 36,
    NotImplemented = 38,
    NotEmpty = 39,
    NotATty = 25,
    MessageTooLarge = 90,
    NotConnected = 128,
}

impl SysError {
    /// Returns the error code for this error.
    pub fn as_code(self) -> u16 {
        self as u16
    }

    /// Decodes an error code into a `SysError` variant.
    pub fn from_code(code: u16) -> Self {
        match code {
            1 => Self::NotPermitted,
            2 => Self::NoEntry,
            3 => Self::NoProcess,
            4 => Self::Interrupted,
            5 => Self::IoError,
            8 => Self::InvalidExecutable,
            9 => Self::BadDescriptor,
            10 => Self::NoChildren,
            11 => Self::ResourceUnavailable,
            12 => Self::OutOfMemory,
            14 => Self::BadAddress,
            17 => Self::AlreadyExists,
            18 => Self::CrossDeviceLink,
            20 => Self::NotDirectory,
            21 => Self::IsDirectory,
            22 => Self::InvalidArgument,
            23 => Self::FileTableFull,
            24 => Self::TooManyFiles,
            28 => Self::NoSpace,
            31 => Self::TooManyLinks,
            32 => Self::BrokenPipe,
            36 => Self::NameTooLong,
            38 => Self::NotImplemented,
            25 => Self::NotATty,
            39 => Self::NotEmpty,
            90 => Self::MessageTooLarge,
            _ => Self::InvalidArgument,
        }
    }
}

impl Display for SysError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SysError::NotPermitted => write!(f, "operation not permitted"),
            SysError::NoEntry => write!(f, "no such entry"),
            SysError::NoProcess => write!(f, "no such process"),
            SysError::Interrupted => write!(f, "interrupted"),
            SysError::IoError => write!(f, "input/output error"),
            SysError::InvalidExecutable => write!(f, "exec format error"),
            SysError::BadDescriptor => write!(f, "bad file descriptor"),
            SysError::NoChildren => write!(f, "no child processes"),
            SysError::ResourceUnavailable => write!(f, "resource temporarily unavailable"),
            SysError::OutOfMemory => write!(f, "cannot allocate memory"),
            SysError::BadAddress => write!(f, "bad address"),
            SysError::AlreadyExists => write!(f, "file exists"),
            SysError::CrossDeviceLink => write!(f, "cross-device link"),
            SysError::NotDirectory => write!(f, "not a directory"),
            SysError::IsDirectory => write!(f, "is a directory"),
            SysError::InvalidArgument => write!(f, "invalid argument"),
            SysError::FileTableFull => write!(f, "too many open files in system"),
            SysError::TooManyFiles => write!(f, "too many open files"),
            SysError::NoSpace => write!(f, "no space left on device"),
            SysError::TooManyLinks => write!(f, "too many links"),
            SysError::BrokenPipe => write!(f, "broken pipe"),
            SysError::NameTooLong => write!(f, "file name too long"),
            SysError::NotImplemented => write!(f, "function not implemented"),
            SysError::NotEmpty => write!(f, "directory not empty"),
            SysError::NotATty => write!(f, "not a tty"),
            SysError::MessageTooLarge => write!(f, "message too large"),
            SysError::NotConnected => write!(f, "socket not connected"),
        }
    }
}

impl From<FsError> for SysError {
    fn from(e: FsError) -> Self {
        match e {
            FsError::OutOfBlock | FsError::OutOfInode => SysError::NoSpace,
            FsError::OutOfFile | FsError::OutOfPipe => SysError::FileTableFull,
            FsError::OutOfRange => SysError::InvalidArgument,
            FsError::Read | FsError::Write => SysError::IoError,
            FsError::Create => SysError::NoSpace,
            FsError::Link => SysError::AlreadyExists,
            FsError::Resolve => SysError::NoEntry,
            FsError::Type => SysError::InvalidArgument,
            FsError::Copy => SysError::BadAddress,
        }
    }
}

impl From<NetError> for SysError {
    fn from(value: NetError) -> Self {
        match value {
            NetError::NotConfigured => SysError::NotPermitted,
            NetError::QueueFull => SysError::ResourceUnavailable,
            NetError::TableFull => SysError::FileTableFull,
            NetError::OutOfSocket => SysError::ResourceUnavailable,
            NetError::PortInUse => SysError::AlreadyExists,
            NetError::BadSocket => SysError::BadDescriptor,
            NetError::InvalidAddress => SysError::InvalidArgument,
            NetError::MalformedPacket => SysError::InvalidArgument,
            NetError::TransmitFailed => SysError::IoError,
            NetError::Interrupted => SysError::Interrupted,
            NetError::RouteNotFound => SysError::NoEntry,
            NetError::PacketTooLarge => SysError::InvalidArgument,
            NetError::ResourceUnavailable => SysError::ResourceUnavailable,
            NetError::InterfaceNotFound => SysError::NoEntry,
            NetError::ChecksumFailed => SysError::InvalidArgument,
            NetError::NotConnected => SysError::NotConnected,
            NetError::ConnectionReset => SysError::NotConnected,
            NetError::AlreadyExists => SysError::AlreadyExists,
            NetError::ConnectionRefused => SysError::NotConnected,
        }
    }
}

impl From<crate::vm::VmError> for SysError {
    fn from(_value: crate::vm::VmError) -> Self {
        SysError::OutOfMemory
    }
}

/// Wrapper for extracting typed syscall arguments from trapframe.
pub struct SyscallArgs<'a> {
    trapframe: &'a TrapFrame,
    proc: &'static Proc,
}

impl<'a> SyscallArgs<'a> {
    /// Creates a new SyscallArgs
    fn new(trapframe: &'a TrapFrame, proc: &'static Proc) -> Self {
        Self { trapframe, proc }
    }

    pub fn proc(&self) -> &Proc {
        self.proc
    }

    /// Returns the argument at the given index as a usize.
    pub fn get_raw(&self, index: usize) -> usize {
        match index {
            0 => self.trapframe.a0,
            1 => self.trapframe.a1,
            2 => self.trapframe.a2,
            3 => self.trapframe.a3,
            4 => self.trapframe.a4,
            5 => self.trapframe.a5,
            _ => panic!("invalid syscall argument index {}", index),
        }
    }

    /// Returns the argument at the given index as an isize.
    pub fn get_int(&self, index: usize) -> isize {
        self.get_raw(index) as isize
    }

    /// Returns the argument at the given index as a virtual address.
    ///
    /// Does not check for legality, since `copyin`/`copyout` will do that.
    pub fn get_addr(&self, index: usize) -> VA {
        VA::from(self.get_raw(index))
    }

    /// Fetch the nth word-sized system call argument as a file descriptor and return both the
    /// descriptor and the corresponding `File`.
    pub fn get_file(&self, index: usize) -> Result<(usize, File), SysError> {
        let fd: usize = try_log!(
            self.get_int(index)
                .try_into()
                .or(Err(SysError::BadDescriptor))
        );

        if fd >= NOFILE {
            err!(SysError::BadDescriptor);
        }

        let data = current_proc().data();
        let files = data.open_files.as_ref().unwrap().files.lock();
        if let Some(file) = &files[fd] {
            return Ok((fd, file.clone()));
        }

        err!(SysError::BadDescriptor);
    }

    /// Fetches a null-terminated string from user space.
    pub fn fetch_string(&self, addr: VA, max: usize) -> Result<String, SysError> {
        let (_proc, data) = current_proc_and_data_mut();

        let mut result = String::with_capacity(max);

        let mut buf = [0u8; 1];
        for i in 0..max {
            try_log!(
                data.pagetable_mut()
                    .copy_from(VA::from(addr.as_usize() + i), &mut buf)
                    .map_err(|_| SysError::BadAddress)
            );

            if buf[0] == 0 {
                return Ok(result);
            }

            result.push(buf[0] as char);
        }

        Ok(result)
    }
}

/// System call numbers
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Fork = 1,
    Exit = 2,
    Wait = 3,
    Pipe = 4,
    Read = 5,
    Kill = 6,
    Exec = 7,
    Fstat = 8,
    Chdir = 9,
    Dup = 10,
    Getpid = 11,
    Sbrk = 12,
    Sleep = 13,
    Uptime = 14,
    Open = 15,
    Write = 16,
    Mknod = 17,
    Unlink = 18,
    Link = 19,
    Mkdir = 20,
    Close = 21,
    Poweroff = 22,
    Ioctl = 23,
    Socket = 24,
    Send = 25,
    Receive = 26,
    Random = 27,
    Lseek = 28,
    Truncate = 29,
    Ftruncate = 30,
    Chmod = 31,
    Fchmod = 32,
    Chown = 33,
    Fchown = 34,
    Access = 35,
    Rename = 36,
    Umask = 37,
    Getuid = 38,
    Geteuid = 39,
    Getgid = 40,
    Getegid = 41,
    Gettimeofday = 42,
    Uname = 43,
    Symlink = 44,
    Readlink = 45,
    Utimensat = 46,
    Alarm = 47,
    Times = 48,
    Sync = 49,
    Getpgrp = 50,
    Setpgid = 75,
    Setsid = 52,
    Nice = 53,
    Dup2 = 54,
    Getppid = 55,
    Setuid = 56,
    Setgid = 57,
    Getpgid = 58,
    Isatty = 59,
    Tcgetattr = 60,
    Tcsetattr = 61,
    Mmap = 62,
    Munmap = 63,
    Mprotect = 64,
    Time = 66,
    Nanosleep = 67,
    ClockGetTime = 68,
    ClockGetRes = 69,
    ClockSetTime = 70,
    Readv = 71,
    Writev = 72,
    Pread = 73,
    Pwrite = 74,
    Getsid = 76,
    Setreuid = 77,
    Setregid = 78,
    Setresuid = 79,
    Setresgid = 80,
    Getresuid = 81,
    Getresgid = 82,
    Mkfifo = 83,
    Pipe2 = 84,
    Ttyname = 85,
    Ttyioctl = 86,
    Tcgetsid = 87,
    Tcflow = 88,
    Tcflush = 89,
    Pathconf = 90,
    Fpathconf = 91,
    Sysconf = 92,
    Confstr = 93,
    Setgroups = 94,
    Getgroups = 95,
    Initgroups = 96,
    Sigaction = 97,
    Sigprocmask = 98,
    Sigpending = 99,
    Sigsuspend = 100,
    Sigreturn = 101,
    Killpg = 102,
    Getenv = 103,
    Setenv = 104,
    Unsetenv = 105,
    Clearenv = 106,
    Getpagesize = 107,
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
    Signalfd4 = 125,
    TimerFdCreate = 126,
    TimerFdSettime = 127,
    TimerFdGettime = 128,
    MemFdCreate = 129,
    PidFdOpen = 130,
    Splice = 131,
    Tee = 132,
    Vmsplice = 133,
    GetRandom = 134,
    CloseRange = 135,
    PrCtl = 136,
    InotifyInit1 = 137,
    InotifyAddWatch = 138,
    InotifyRmWatch = 139,
    SetNs = 140,
    Unshare = 141,
    CapGet = 142,
    CapSet = 143,
    Seccomp = 144,
    PivotRoot = 145,
    Sethostname = 146,
    Gethostname = 147,
    OverlayMount = 148,
    OverlayUmount = 149,
    NsOpen = 150,
}

impl TryFrom<usize> for Syscall {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Syscall::Fork),
            2 => Ok(Syscall::Exit),
            3 => Ok(Syscall::Wait),
            4 => Ok(Syscall::Pipe),
            5 => Ok(Syscall::Read),
            6 => Ok(Syscall::Kill),
            7 => Ok(Syscall::Exec),
            8 => Ok(Syscall::Fstat),
            9 => Ok(Syscall::Chdir),
            10 => Ok(Syscall::Dup),
            11 => Ok(Syscall::Getpid),
            12 => Ok(Syscall::Sbrk),
            13 => Ok(Syscall::Sleep),
            14 => Ok(Syscall::Uptime),
            15 => Ok(Syscall::Open),
            16 => Ok(Syscall::Write),
            17 => Ok(Syscall::Mknod),
            18 => Ok(Syscall::Unlink),
            19 => Ok(Syscall::Link),
            20 => Ok(Syscall::Mkdir),
            21 => Ok(Syscall::Close),
            22 => Ok(Syscall::Poweroff),
            23 => Ok(Syscall::Ioctl),
            24 => Ok(Syscall::Socket),
            25 => Ok(Syscall::Send),
            26 => Ok(Syscall::Receive),
            27 => Ok(Syscall::Random),
            28 => Ok(Syscall::Lseek),
            29 => Ok(Syscall::Truncate),
            30 => Ok(Syscall::Ftruncate),
            31 => Ok(Syscall::Chmod),
            32 => Ok(Syscall::Fchmod),
            33 => Ok(Syscall::Chown),
            34 => Ok(Syscall::Fchown),
            35 => Ok(Syscall::Access),
            36 => Ok(Syscall::Rename),
            37 => Ok(Syscall::Umask),
            38 => Ok(Syscall::Getuid),
            39 => Ok(Syscall::Geteuid),
            40 => Ok(Syscall::Getgid),
            41 => Ok(Syscall::Getegid),
            42 => Ok(Syscall::Gettimeofday),
            43 => Ok(Syscall::Uname),
            44 => Ok(Syscall::Symlink),
            45 => Ok(Syscall::Readlink),
            46 => Ok(Syscall::Utimensat),
            47 => Ok(Syscall::Alarm),
            48 => Ok(Syscall::Times),
            49 => Ok(Syscall::Sync),
            50 => Ok(Syscall::Getpgrp),
            51 => Err(SysError::NotImplemented),
            52 => Ok(Syscall::Setsid),
            53 => Ok(Syscall::Nice),
            54 => Ok(Syscall::Dup2),
            55 => Ok(Syscall::Getppid),
            56 => Ok(Syscall::Setuid),
            57 => Ok(Syscall::Setgid),
            58 => Ok(Syscall::Getpgid),
            59 => Ok(Syscall::Isatty),
            60 => Ok(Syscall::Tcgetattr),
            61 => Ok(Syscall::Tcsetattr),
            62 => Ok(Syscall::Mmap),
            63 => Ok(Syscall::Munmap),
            64 => Ok(Syscall::Mprotect),
            66 => Ok(Syscall::Time),
            67 => Ok(Syscall::Nanosleep),
            68 => Ok(Syscall::ClockGetTime),
            69 => Ok(Syscall::ClockGetRes),
            70 => Ok(Syscall::ClockSetTime),
            71 => Ok(Syscall::Readv),
            72 => Ok(Syscall::Writev),
            73 => Ok(Syscall::Pread),
            74 => Ok(Syscall::Pwrite),
            75 => Ok(Syscall::Setpgid),
            76 => Ok(Syscall::Getsid),
            77 => Ok(Syscall::Setreuid),
            78 => Ok(Syscall::Setregid),
            79 => Ok(Syscall::Setresuid),
            80 => Ok(Syscall::Setresgid),
            81 => Ok(Syscall::Getresuid),
            82 => Ok(Syscall::Getresgid),
            83 => Ok(Syscall::Mkfifo),
            84 => Ok(Syscall::Pipe2),
            85 => Ok(Syscall::Ttyname),
            86 => Ok(Syscall::Ttyioctl),
            87 => Ok(Syscall::Tcgetsid),
            88 => Ok(Syscall::Tcflow),
            89 => Ok(Syscall::Tcflush),
            90 => Ok(Syscall::Pathconf),
            91 => Ok(Syscall::Fpathconf),
            92 => Ok(Syscall::Sysconf),
            93 => Ok(Syscall::Confstr),
            94 => Ok(Syscall::Setgroups),
            95 => Ok(Syscall::Getgroups),
            96 => Ok(Syscall::Initgroups),
            97 => Ok(Syscall::Sigaction),
            98 => Ok(Syscall::Sigprocmask),
            99 => Ok(Syscall::Sigpending),
            100 => Ok(Syscall::Sigsuspend),
            101 => Ok(Syscall::Sigreturn),
            102 => Ok(Syscall::Killpg),
            103 => Ok(Syscall::Getenv),
            104 => Ok(Syscall::Setenv),
            105 => Ok(Syscall::Unsetenv),
            106 => Ok(Syscall::Clearenv),
            107 => Ok(Syscall::Getpagesize),
            108 => Ok(Syscall::TcpSocket),
            109 => Ok(Syscall::TcpBind),
            110 => Ok(Syscall::TcpListen),
            111 => Ok(Syscall::TcpAccept),
            112 => Ok(Syscall::TcpConnect),
            113 => Ok(Syscall::TcpSend),
            114 => Ok(Syscall::TcpRecv),
            115 => Ok(Syscall::Fcntl),
            116 => Ok(Syscall::Poll),
            117 => Ok(Syscall::EpollCreate1),
            118 => Ok(Syscall::EpollCtl),
            119 => Ok(Syscall::EpollWait),
            120 => Ok(Syscall::Clone),
            121 => Ok(Syscall::Gettid),
            122 => Ok(Syscall::ExitGroup),
            123 => Ok(Syscall::Futex),
            124 => Ok(Syscall::EventFd2),
            125 => Ok(Syscall::Signalfd4),
            126 => Ok(Syscall::TimerFdCreate),
            127 => Ok(Syscall::TimerFdSettime),
            128 => Ok(Syscall::TimerFdGettime),
            129 => Ok(Syscall::MemFdCreate),
            130 => Ok(Syscall::PidFdOpen),
            131 => Ok(Syscall::Splice),
            132 => Ok(Syscall::Tee),
            133 => Ok(Syscall::Vmsplice),
            134 => Ok(Syscall::GetRandom),
            135 => Ok(Syscall::CloseRange),
            136 => Ok(Syscall::PrCtl),
            137 => Ok(Syscall::InotifyInit1),
            138 => Ok(Syscall::InotifyAddWatch),
            139 => Ok(Syscall::InotifyRmWatch),
            140 => Ok(Syscall::SetNs),
            141 => Ok(Syscall::Unshare),
             142 => Ok(Syscall::CapGet),
             143 => Ok(Syscall::CapSet),
             144 => Ok(Syscall::Seccomp),
              145 => Ok(Syscall::PivotRoot),
              146 => Ok(Syscall::Sethostname),
              147 => Ok(Syscall::Gethostname),
              148 => Ok(Syscall::OverlayMount),
               149 => Ok(Syscall::OverlayUmount),
               150 => Ok(Syscall::NsOpen),
             _ => Err(SysError::NotImplemented),
        }
    }
}

/// Handle a system call.
///
/// # Safety
/// Called from `usertrap` in `trap.rs`.
#[unsafe(no_mangle)]
pub unsafe fn syscall(trapframe: &mut TrapFrame) {
    let proc = current_proc();
    let args = SyscallArgs::new(trapframe, proc);

    // Seccomp filter check (skip for seccomp syscall itself)
    if trapframe.a7 != 144 {
        match crate::seccomp::seccomp_check(trapframe.a7) {
            crate::seccomp::SeccompAction::Kill => {
                crate::proc::exit(-1);
            }
            crate::seccomp::SeccompAction::Allow => {}
        }
    }

    let result = match Syscall::try_from(trapframe.a7) {
        Ok(syscall) => match syscall {
            Syscall::Fork => sys_fork(&args),
            Syscall::Exit => sys_exit(&args),
            Syscall::Wait => sys_wait(&args),
            Syscall::Pipe => sys_pipe(&args),
            Syscall::Read => sys_read(&args),
            Syscall::Kill => sys_kill(&args),
            Syscall::Exec => sys_exec(&args),
            Syscall::Fstat => sys_fstat(&args),
            Syscall::Chdir => sys_chdir(&args),
            Syscall::Dup => sys_dup(&args),
            Syscall::Getpid => sys_getpid(&args),
            Syscall::Sbrk => sys_sbrk(&args),
            Syscall::Sleep => sys_sleep(&args),
            Syscall::Uptime => sys_uptime(&args),
            Syscall::Open => sys_open(&args),
            Syscall::Write => sys_write(&args),
            Syscall::Mknod => sys_mknod(&args),
            Syscall::Unlink => sys_unlink(&args),
            Syscall::Link => sys_link(&args),
            Syscall::Mkdir => sys_mkdir(&args),
            Syscall::Close => sys_close(&args),
            Syscall::Poweroff => sys_poweroff(&args),
            Syscall::Ioctl => sys_ioctl(&args),
            Syscall::Socket => sys_socket(&args),
            Syscall::Send => sys_send(&args),
            Syscall::Receive => sys_receive(&args),
            Syscall::Random => sys_random(&args),
            Syscall::Lseek => sys_lseek(&args),
            Syscall::Truncate => sys_truncate(&args),
            Syscall::Ftruncate => sys_ftruncate(&args),
            Syscall::Chmod => sys_chmod(&args),
            Syscall::Fchmod => sys_fchmod(&args),
            Syscall::Chown => sys_chown(&args),
            Syscall::Fchown => sys_fchown(&args),
            Syscall::Access => sys_access(&args),
            Syscall::Rename => sys_rename(&args),
            Syscall::Umask => sys_umask(&args),
            Syscall::Getuid => sys_getuid(&args),
            Syscall::Geteuid => sys_geteuid(&args),
            Syscall::Getgid => sys_getgid(&args),
            Syscall::Getegid => sys_getegid(&args),
            Syscall::Gettimeofday => sys_gettimeofday(&args),
            Syscall::Uname => sys_uname(&args),
            Syscall::Symlink => sys_symlink(&args),
            Syscall::Readlink => sys_readlink(&args),
            Syscall::Utimensat => sys_utimensat(&args),
            Syscall::Alarm => sys_alarm(&args),
            Syscall::Times => sys_times(&args),
            Syscall::Sync => sys_sync(&args),
            Syscall::Getpgrp => sys_getpgrp(&args),
            Syscall::Setpgid => sys_setpgid(&args),
            Syscall::Setsid => sys_setsid(&args),
            Syscall::Nice => sys_nice(&args),
            Syscall::Dup2 => sys_dup2(&args),
            Syscall::Getppid => sys_getppid(&args),
            Syscall::Setuid => sys_setuid(&args),
            Syscall::Setgid => sys_setgid(&args),
            Syscall::Getpgid => sys_getpgid(&args),
            Syscall::Isatty => sys_isatty(&args),
            Syscall::Tcgetattr => sys_tcgetattr(&args),
            Syscall::Tcsetattr => sys_tcsetattr(&args),
            Syscall::Mmap => sys_mmap(&args),
            Syscall::Munmap => sys_munmap(&args),
            Syscall::Mprotect => sys_mprotect(&args),
            Syscall::Time => sys_time(&args),
            Syscall::Nanosleep => sys_nanosleep(&args),
            Syscall::ClockGetTime => sys_clock_gettime(&args),
            Syscall::ClockGetRes => sys_clock_getres(&args),
            Syscall::ClockSetTime => sys_clock_settime(&args),
            Syscall::Readv => sys_readv(&args),
            Syscall::Writev => sys_writev(&args),
            Syscall::Pread => sys_pread(&args),
            Syscall::Pwrite => sys_pwrite(&args),
            Syscall::Getsid => sys_getsid(&args),
            Syscall::Setreuid => sys_setreuid(&args),
            Syscall::Setregid => sys_setregid(&args),
            Syscall::Setresuid => sys_setresuid(&args),
            Syscall::Setresgid => sys_setresgid(&args),
            Syscall::Getresuid => sys_getresuid(&args),
            Syscall::Getresgid => sys_getresgid(&args),
            Syscall::Mkfifo => sys_mkfifo(&args),
            Syscall::Pipe2 => sys_pipe2(&args),
            Syscall::Ttyname => sys_ttyname(&args),
            Syscall::Ttyioctl => sys_ttyioctl(&args),
            Syscall::Tcgetsid => sys_tcgetsid(&args),
            Syscall::Tcflow => sys_tcflow(&args),
            Syscall::Tcflush => sys_tcflush(&args),
            Syscall::Pathconf => sys_pathconf(&args),
            Syscall::Fpathconf => sys_fpathconf(&args),
            Syscall::Sysconf => sys_sysconf(&args),
            Syscall::Confstr => sys_confstr(&args),
            Syscall::Setgroups => sys_setgroups(&args),
            Syscall::Getgroups => sys_getgroups(&args),
            Syscall::Initgroups => sys_initgroups(&args),
            Syscall::Sigaction => sys_sigaction(&args),
            Syscall::Sigprocmask => sys_sigprocmask(&args),
            Syscall::Sigpending => sys_sigpending(&args),
            Syscall::Sigsuspend => sys_sigsuspend(&args),
            Syscall::Sigreturn => sys_sigreturn(&args),
            Syscall::Killpg => sys_killpg(&args),
            Syscall::Getenv => sys_getenv(&args),
            Syscall::Setenv => sys_setenv(&args),
            Syscall::Unsetenv => sys_unsetenv(&args),
            Syscall::Clearenv => sys_clearenv(&args),
            Syscall::Getpagesize => sys_getpagesize(&args),
            Syscall::TcpSocket => sys_tcp_socket(&args),
            Syscall::TcpBind => sys_tcp_bind(&args),
            Syscall::TcpListen => sys_tcp_listen(&args),
            Syscall::TcpAccept => sys_tcp_accept(&args),
            Syscall::TcpConnect => sys_tcp_connect(&args),
            Syscall::TcpSend => sys_tcp_send(&args),
            Syscall::TcpRecv => sys_tcp_recv(&args),
            Syscall::Fcntl => sys_fcntl(&args),
            Syscall::Poll => sys_poll(&args),
            Syscall::EpollCreate1 => sys_epoll_create1(&args),
            Syscall::EpollCtl => sys_epoll_ctl(&args),
            Syscall::EpollWait => sys_epoll_wait(&args),
            Syscall::Clone => sys_clone(&args),
            Syscall::Gettid => sys_gettid(&args),
            Syscall::ExitGroup => sys_exit_group(&args),
            Syscall::Futex => sys_futex(&args),
            Syscall::EventFd2 => sys_eventfd2(&args),
            Syscall::Signalfd4 => sys_signalfd4(&args),
            Syscall::TimerFdCreate => sys_timerfd_create(&args),
            Syscall::TimerFdSettime => sys_timerfd_settime(&args),
            Syscall::TimerFdGettime => sys_timerfd_gettime(&args),
            Syscall::MemFdCreate => sys_memfd_create(&args),
            Syscall::PidFdOpen => sys_pidfd_open(&args),
            Syscall::Splice => sys_splice(&args),
            Syscall::Tee => sys_tee(&args),
            Syscall::Vmsplice => sys_vmsplice(&args),
            Syscall::GetRandom => sys_getrandom(&args),
            Syscall::CloseRange => sys_close_range(&args),
            Syscall::PrCtl => sys_prctl(&args),
            Syscall::InotifyInit1 => sys_inotify_init1(&args),
            Syscall::InotifyAddWatch => sys_inotify_add_watch(&args),
            Syscall::InotifyRmWatch => sys_inotify_rm_watch(&args),
            Syscall::SetNs => sys_setns(&args),
            Syscall::Unshare => sys_unshare(&args),
            Syscall::PivotRoot => sys_pivot_root(&args),
            Syscall::Sethostname => sys_sethostname(&args),
            Syscall::Gethostname => sys_gethostname(&args),
            Syscall::CapGet => sys_capget(&args),
            Syscall::CapSet => sys_capset(&args),
            Syscall::Seccomp => sys_seccomp(&args),
            Syscall::OverlayMount => sys_overlay_mount(&args),
            Syscall::OverlayUmount => sys_overlay_umount(&args),
            Syscall::NsOpen => sys_nsopen(&args),
        },
        Err(e) => Err(e),
    };

    trapframe.a0 = match log!(result) {
        Ok(v) => v,
        Err(error) => {
            #[cfg(debug_assertions)]
            {
                let pid = *proc.inner.lock().pid;
                println!(
                    "! syscall error ({}) from proc {} ({})",
                    error,
                    pid,
                    proc.data().name,
                );
            }
            (-(error.as_code() as isize)) as usize
        }
    };
}
