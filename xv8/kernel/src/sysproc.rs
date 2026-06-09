use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::fs::{FsError, InodeType, Path};
use crate::memlayout::QEMU_POWER;
use crate::param::MMAP_BASE;
use crate::proc::{self, Channel, Pid, ProcState, current_proc, current_proc_and_data_mut, wakeup_n};
use crate::rng::rand_bytes;
use crate::riscv::{pg_round_up, PGSIZE};
use crate::signal;
use crate::syscall::{SysError, SyscallArgs};
use crate::trap::TICKS;
use crate::vm::VA;

pub fn sys_exit(args: &SyscallArgs) -> ! {
    let n = args.get_int(0);
    proc::exit(n);
}

pub fn sys_getpid(args: &SyscallArgs) -> Result<usize, SysError> {
    let tgid = args.proc().inner.lock().tgid;
    Ok(*tgid)
}

pub fn sys_fork(_args: &SyscallArgs) -> Result<usize, SysError> {
    match log!(proc::fork()) {
        Ok(pid) => Ok(*pid),
        Err(_) => Err(SysError::ResourceUnavailable),
    }
}

pub fn sys_wait(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);
    match proc::wait(addr) {
        Some(pid) => Ok(*pid),
        None => err!(SysError::NoChildren),
    }
}

pub fn sys_sbrk(args: &SyscallArgs) -> Result<usize, SysError> {
    let size = args.get_int(0);
    let addr = args.proc().data().size;

    match unsafe { log!(proc::grow(size, size >= 0)) } {
        Ok(_) => Ok(addr),
        Err(_) => Err(SysError::OutOfMemory),
    }
}

pub fn sys_sleep(args: &SyscallArgs) -> Result<usize, SysError> {
    let duration = args.get_int(0).max(0) as usize;

    let mut ticks = TICKS.lock();
    let ticks0 = *ticks;

    while *ticks - ticks0 < duration {
        if current_proc().is_killed() {
            return Err(SysError::Interrupted);
        }

        ticks = proc::sleep(Channel::Ticks, ticks);
    }

    Ok(0)
}

pub fn sys_kill(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0);

    if proc::kill(unsafe { Pid::from_usize(pid as usize) }) {
        Ok(0)
    } else {
        Err(SysError::NoProcess)
    }
}

pub fn sys_uptime(_args: &SyscallArgs) -> Result<usize, SysError> {
    let ticks = *TICKS.lock();
    Ok(ticks)
}

pub fn sys_random(args: &SyscallArgs) -> Result<usize, SysError> {
    let dest_buf = args.get_addr(0);
    let len = args.get_int(1) as usize;

    let mut src_buf = vec![0u8; len];
    rand_bytes(&mut src_buf);

    try_log!(proc::copy_to_user(&src_buf, dest_buf).map_err(|_| SysError::BadAddress));

    Ok(0)
}

pub fn sys_poweroff(args: &SyscallArgs) -> ! {
    let code = match args.get_int(0) as u32 {
        0 => 0x5555,
        c => (c << 16) | 0x3333,
    };

    println!("! powering off...");

    unsafe { *(QEMU_POWER as *mut u32) = code };

    unreachable!("poweroff failed");
}

pub fn sys_umask(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0o022)
}

pub fn sys_getuid(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

pub fn sys_geteuid(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

pub fn sys_getgid(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

pub fn sys_getegid(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

#[repr(C)]
pub struct TimeVal {
    pub sec: u32,
    pub usec: u32,
}

pub fn sys_gettimeofday(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);

    let now = signal::get_time_us();
    let tv = TimeVal {
        sec: (now / 1_000_000) as u32,
        usec: (now % 1_000_000) as u32,
    };

    let src = unsafe {
        core::slice::from_raw_parts(&tv as *const _ as *const u8, core::mem::size_of::<TimeVal>())
    };
    try_log!(proc::copy_to_user(src, addr).map_err(|_| SysError::BadAddress));

    Ok(0)
}

#[repr(C)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

pub fn sys_uname(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);

    let mut uts = Utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };

    let sysname = b"xv8";
    let release = b"1.0.0";
    let version = b"#1 SMP";
    let machine = b"riscv64";

    uts.sysname[..sysname.len()].copy_from_slice(sysname);
    uts.release[..release.len()].copy_from_slice(release);
    uts.version[..version.len()].copy_from_slice(version);
    uts.machine[..machine.len()].copy_from_slice(machine);

    let src = unsafe {
        core::slice::from_raw_parts(&uts as *const _ as *const u8, core::mem::size_of::<Utsname>())
    };
    try_log!(proc::copy_to_user(src, addr).map_err(|_| SysError::BadAddress));

    Ok(0)
}

pub fn sys_alarm(args: &SyscallArgs) -> Result<usize, SysError> {
    let seconds = args.get_int(0) as usize;

    let (_proc, data) = current_proc_and_data_mut();

    let old_alarm = data.signals.get_alarm_time();
    let now = signal::get_time_ms();

    if seconds == 0 {
        data.signals.set_alarm_time(0);
    } else {
        data.signals.set_alarm_time(now + seconds * 1000);
    }

    if old_alarm > now {
        Ok((old_alarm - now + 999) / 1000)
    } else {
        Ok(0)
    }
}

#[repr(C)]
pub struct Tms {
    pub utime: u32,
    pub stime: u32,
    pub cutime: u32,
    pub cstime: u32,
}

pub fn sys_times(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);

    let (_proc, data) = current_proc_and_data_mut();

    let tms = Tms {
        utime: data.utime as u32,
        stime: data.stime as u32,
        cutime: 0,
        cstime: 0,
    };

    let src = unsafe {
        core::slice::from_raw_parts(&tms as *const _ as *const u8, core::mem::size_of::<Tms>())
    };
    try_log!(proc::copy_to_user(src, addr).map_err(|_| SysError::BadAddress));

    Ok(0)
}

pub fn sys_sync(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

pub fn sys_getpgrp(_args: &SyscallArgs) -> Result<usize, SysError> {
    let pgrp = current_proc().data().pgrp;
    Ok(*pgrp)
}

pub fn sys_setpgid(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0) as usize;
    let pgid = args.get_int(1) as usize;

    let (current_proc, data) = current_proc_and_data_mut();
    let current_pid = *current_proc.inner.lock().pid;

    if pid == 0 || pid == current_pid {
        if pgid == 0 {
            // SAFETY: current_pid is a valid allocated PID
            data.pgrp = unsafe { Pid::from_usize(current_pid) };
        } else {
            // SAFETY: pgid is assumed to be a valid PID
            data.pgrp = unsafe { Pid::from_usize(pgid) };
        }
        Ok(0)
    } else {
        err!(SysError::NoProcess)
    }
}

pub fn sys_setsid(_args: &SyscallArgs) -> Result<usize, SysError> {
    let (proc, data) = current_proc_and_data_mut();
    let pid = *proc.inner.lock().pid;

    // SAFETY: pid is a valid allocated PID
    data.pgrp = unsafe { Pid::from_usize(pid) };

    Ok(pid)
}

pub fn sys_nice(args: &SyscallArgs) -> Result<usize, SysError> {
    let incr = args.get_int(0) as i32;

    let (_proc, data) = current_proc_and_data_mut();

    let new_nice = (data.nice + incr).clamp(-20, 19);
    data.nice = new_nice;

    Ok(new_nice as usize)
}

pub fn sys_getppid(_args: &SyscallArgs) -> Result<usize, SysError> {
    let proc = current_proc();
    let parents = proc::PROC_TABLE.parents.lock();
    let parent_id = parents[proc.id].unwrap_or(0);
    Ok(parent_id)
}

pub fn sys_setuid(args: &SyscallArgs) -> Result<usize, SysError> {
    let uid = args.get_int(0) as u32;

    let (_proc, data) = current_proc_and_data_mut();
    data.uid = uid;

    Ok(0)
}

pub fn sys_setgid(args: &SyscallArgs) -> Result<usize, SysError> {
    let gid = args.get_int(0) as u32;

    let (_proc, data) = current_proc_and_data_mut();
    data.gid = gid;

    Ok(0)
}

pub fn sys_getpgid(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0) as usize;

    if pid == 0 {
        return Ok(*current_proc().data().pgrp);
    }

    let current_proc = current_proc();
    let current_pid = *current_proc.inner.lock().pid;

    if pid == current_pid {
        return Ok(*current_proc.data().pgrp);
    }

    let current_id = current_proc.id;
    let parents = proc::PROC_TABLE.parents.lock();

    for p in proc::PROC_TABLE.iter() {
        if parents[p.id] == Some(current_id) {
            if *p.inner.lock().pid == *unsafe { Pid::from_usize(pid) } {
                return Ok(*p.data().pgrp);
            }
        }
    }

    err!(SysError::NoProcess)
}

pub fn sys_isatty(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));

    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE as usize {
            return Ok(1);
        }
    }

    Ok(0)
}

pub fn sys_tcgetattr(args: &SyscallArgs) -> Result<usize, SysError> {
    let _fd = args.get_int(0) as usize;
    let _addr = args.get_addr(1);

    Ok(0)
}

pub fn sys_tcsetattr(args: &SyscallArgs) -> Result<usize, SysError> {
    let _fd = args.get_int(0) as usize;
    let _addr = args.get_addr(1);
    let _opt = args.get_int(2) as usize;

    Ok(0)
}

#[repr(C)]
pub struct Itimerval {
    pub interval: TimeVal,
    pub value: TimeVal,
}

pub fn sys_getitimer(args: &SyscallArgs) -> Result<usize, SysError> {
    let _which = args.get_int(0) as u32;
    let _addr = args.get_addr(1);
    Ok(0)
}

pub fn sys_setitimer(args: &SyscallArgs) -> Result<usize, SysError> {
    let _which = args.get_int(0) as u32;
    let _addr = args.get_addr(1);
    let _old_addr = args.get_addr(2);
    Ok(0)
}

pub fn sys_mmap(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);
    let length = args.get_int(1) as usize;
    let _prot = args.get_int(2) as usize;
    let flags = args.get_int(3) as usize;
    let _fd = args.get_int(4) as isize;
    let _offset = args.get_int(5) as usize;

    if length == 0 {
        err!(SysError::InvalidArgument);
    }

    if flags & 0x2 == 0 {
        err!(SysError::InvalidArgument);
    }

    if addr.as_usize() == 0 {
        let length = pg_round_up(length);
        let (proc, data) = current_proc_and_data_mut();
        let mut mmap_next = data.mmap_next;
        if mmap_next.as_usize() < length {
            err!(SysError::OutOfMemory);
        }
        let new_mmap_next = VA::from(mmap_next.as_usize() - length);
        data.mmap_next = new_mmap_next;
        Ok(new_mmap_next.as_usize())
    } else {
        err!(SysError::NotImplemented)
    }
}

pub fn sys_munmap(args: &SyscallArgs) -> Result<usize, SysError> {
    let _addr = args.get_addr(0);
    let _length = args.get_int(1) as usize;
    Ok(0)
}

pub fn sys_mprotect(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(0)
}

pub fn sys_time(args: &SyscallArgs) -> Result<usize, SysError> {
    let t = *TICKS.lock();
    Ok(t / 100) // TICKS 以 10ms 为单位，转为秒
}

pub fn sys_nanosleep(args: &SyscallArgs) -> Result<usize, SysError> {
    let req_addr = args.get_addr(0);
    let _rem_addr = args.get_addr(1);

    let mut buf = [0u8; 16];
    let (_proc, data) = current_proc_and_data_mut();
    if data.pagetable_mut().copy_from(req_addr, &mut buf).is_err() {
        err!(SysError::BadAddress);
    }

    let sec = u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]]);
    let nsec = u64::from_le_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]);

    let total_ticks = (sec * 100) as usize + (nsec / 10_000_000) as usize;

    let mut ticks = TICKS.lock();
    let ticks0 = *ticks;

    while *ticks - ticks0 < total_ticks {
        if current_proc().is_killed() {
            return Err(SysError::Interrupted);
        }
        ticks = proc::sleep(Channel::Ticks, ticks);
    }

    Ok(0)
}

pub fn sys_clock_gettime(args: &SyscallArgs) -> Result<usize, SysError> {
    let _clock_id = args.get_int(0) as usize;
    let ts_addr = args.get_addr(1);

    let t = *TICKS.lock();

    let sec = (t / 100) as u64;
    let nsec = ((t % 100) * 10_000_000) as u64;
    let mut ts = [0u8; 16];
    ts[..8].copy_from_slice(&sec.to_le_bytes());
    ts[8..].copy_from_slice(&nsec.to_le_bytes());

    let (_proc, data) = current_proc_and_data_mut();
    let _ = data.pagetable_mut().copy_to(&ts, ts_addr);

    Ok(0)
}

pub fn sys_clock_getres(args: &SyscallArgs) -> Result<usize, SysError> {
    let _clock_id = args.get_int(0) as usize;
    let ts_addr = args.get_addr(1);

    let ts = [0u64.to_le_bytes(), 100u64.to_le_bytes()].concat(); // 0 sec, 100 nsec resolution

    let (_proc, data) = current_proc_and_data_mut();
    let _ = data.pagetable_mut().copy_to(&ts, ts_addr);

    Ok(0)
}

pub fn sys_clock_settime(args: &SyscallArgs) -> Result<usize, SysError> {
    let _clock_id = args.get_int(0) as usize;
    let _ts_addr = args.get_addr(1);
    Ok(0)
}

pub fn sys_getsid(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0) as usize;

    if pid == 0 {
        let current_pid = *args.proc().inner.lock().pid;
        return Ok(current_pid);
    }

    // simplified: return the given pid as its own session ID
    Ok(pid)
}

pub fn sys_setreuid(args: &SyscallArgs) -> Result<usize, SysError> {
    let ruid = args.get_int(0) as u32;
    let euid = args.get_int(1) as u32;

    let (_, data) = current_proc_and_data_mut();

    if ruid != u32::MAX {
        data.uid = ruid;
    }
    if euid != u32::MAX {
        data.euid = euid;
        data.suid = euid;
    }

    Ok(0)
}

pub fn sys_setregid(args: &SyscallArgs) -> Result<usize, SysError> {
    let rgid = args.get_int(0) as u32;
    let egid = args.get_int(1) as u32;

    let (_, data) = current_proc_and_data_mut();

    if rgid != u32::MAX {
        data.gid = rgid;
    }
    if egid != u32::MAX {
        data.egid = egid;
        data.sgid = egid;
    }

    Ok(0)
}

pub fn sys_setresuid(args: &SyscallArgs) -> Result<usize, SysError> {
    let ruid = args.get_int(0) as u32;
    let euid = args.get_int(1) as u32;
    let suid = args.get_int(2) as u32;

    let (_, data) = current_proc_and_data_mut();

    if ruid != u32::MAX {
        data.uid = ruid;
    }
    if euid != u32::MAX {
        data.euid = euid;
    }
    if suid != u32::MAX {
        data.suid = suid;
    }

    Ok(0)
}

pub fn sys_setresgid(args: &SyscallArgs) -> Result<usize, SysError> {
    let rgid = args.get_int(0) as u32;
    let egid = args.get_int(1) as u32;
    let sgid = args.get_int(2) as u32;

    let (_, data) = current_proc_and_data_mut();

    if rgid != u32::MAX {
        data.gid = rgid;
    }
    if egid != u32::MAX {
        data.egid = egid;
    }
    if sgid != u32::MAX {
        data.sgid = sgid;
    }

    Ok(0)
}

pub fn sys_getresuid(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);

    let (_, data) = current_proc_and_data_mut();

    let ruid = data.uid;
    let euid = data.euid;
    let suid = data.suid;

    let buf = [
        ruid.to_le_bytes(),
        euid.to_le_bytes(),
        suid.to_le_bytes(),
    ].concat();

    data.pagetable_mut().copy_to(&buf, addr).map_err(|_| SysError::BadAddress)?;

    Ok(0)
}

pub fn sys_getresgid(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);

    let (_, data) = current_proc_and_data_mut();

    let rgid = data.gid;
    let egid = data.egid;
    let sgid = data.sgid;

    let buf = [
        rgid.to_le_bytes(),
        egid.to_le_bytes(),
        sgid.to_le_bytes(),
    ].concat();

    data.pagetable_mut().copy_to(&buf, addr).map_err(|_| SysError::BadAddress)?;

    Ok(0)
}

pub fn sys_ttyname(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));
    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE {
            let buf_addr = args.get_addr(1);
            let buf_len = args.get_int(2) as usize;
            let name = b"/dev/console\0";
            let len = name.len().min(buf_len);
            let (_proc, data) = current_proc_and_data_mut();
            data.pagetable_mut()
                .copy_to(&name[..len], buf_addr)
                .map_err(|_| SysError::BadAddress)?;
            return Ok(len);
        }
    }

    err!(SysError::NotATty)
}

pub fn sys_ttyioctl(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));
    let ioctl_cmd = args.get_int(1) as usize;
    let ioctl_arg = args.get_int(2) as usize;

    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE {
            drop(file_inner);
            return crate::console::Console::ioctl(ioctl_cmd, ioctl_arg);
        }
    }

    err!(SysError::NotATty)
}

pub fn sys_tcgetsid(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_, file) = try_log!(args.get_file(0));
    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE {
            if let Some(pid) = crate::console::Console::foreground_pid() {
                return Ok(*pid);
            }
            return Ok(0);
        }
    }

    err!(SysError::NotATty)
}

pub fn sys_tcflow(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_proc, file) = try_log!(args.get_file(0));
    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE {
            return Ok(0);
        }
    }

    err!(SysError::NotATty)
}

pub fn sys_tcflush(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd = args.get_int(0) as usize;

    if fd >= crate::param::NOFILE {
        err!(SysError::BadDescriptor);
    }

    let (_proc, file) = try_log!(args.get_file(0));
    let file_inner = crate::file::FILE_TABLE.inner[file.id].lock();

    if let crate::file::FileType::Device { major, .. } = &file_inner.r#type {
        if *major as usize == crate::file::CONSOLE {
            return Ok(0);
        }
    }

    err!(SysError::NotATty)
}

pub fn sys_pathconf(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = args.get_int(1) as usize;
    let _path = try_log!(args.fetch_string(args.get_addr(0), crate::param::MAXPATH));
    match name {
        1 => Ok(65535),   // _PC_LINK_MAX
        2 => Ok(255),     // _PC_MAX_CANON
        3 => Ok(255),     // _PC_MAX_INPUT
        4 => Ok(255),     // _PC_NAME_MAX
        5 => Ok(4096),    // _PC_PATH_MAX
        6 => Ok(4096),    // _PC_PIPE_BUF
        7 => Ok(1),       // _PC_CHOWN_RESTRICTED
        8 => Ok(1),       // _PC_NO_TRUNC
        9 => Ok(0),       // _PC_VDISABLE
        _ => err!(SysError::InvalidArgument),
    }
}

pub fn sys_fpathconf(args: &SyscallArgs) -> Result<usize, SysError> {
    let _fd = args.get_int(0) as usize;
    let name = args.get_int(1) as usize;
    match name {
        1 => Ok(65535),
        2 => Ok(255),
        3 => Ok(255),
        4 => Ok(255),
        5 => Ok(4096),
        6 => Ok(4096),
        7 => Ok(1),
        8 => Ok(1),
        9 => Ok(0),
        _ => err!(SysError::InvalidArgument),
    }
}

pub fn sys_sysconf(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = args.get_int(0) as usize;
    match name {
        1 => Ok(131072),  // _SC_ARG_MAX
        2 => Ok(64),      // _SC_CHILD_MAX
        3 => Ok(100),     // _SC_CLK_TCK
        4 => Ok(16),      // _SC_NGROUPS_MAX
        5 => Ok(256),     // _SC_OPEN_MAX
        6 => Ok(4096),    // _SC_PAGESIZE
        7 => Ok(256),     // _SC_STREAM_MAX
        8 => Ok(6),       // _SC_TZNAME_MAX
        9 => Ok(1),       // _SC_JOB_CONTROL
        10 => Ok(1),      // _SC_SAVED_IDS
        11 => Ok(200809), // _SC_VERSION
        12 => Ok(64),     // _SC_HOST_NAME_MAX
        _ => err!(SysError::InvalidArgument),
    }
}

pub fn sys_setgroups(args: &SyscallArgs) -> Result<usize, SysError> {
    let size = args.get_int(1) as usize;
    if size > 16 {
        err!(SysError::InvalidArgument)
    }
    let list_addr = args.get_addr(0);
    let (_proc, data) = current_proc_and_data_mut();
    if size > 0 {
        let mut groups = [0u32; 16];
        for i in 0..size {
            let addr = VA::new(list_addr.as_usize() + i * 4);
            let mut buf = [0u8; 4];
            data.pagetable_mut()
                .copy_from(addr, &mut buf)
                .map_err(|_| SysError::BadAddress)?;
            groups[i] = u32::from_ne_bytes(buf);
        }
        data.groups = groups;
    }
    data.ngroups = size;
    Ok(0)
}

pub fn sys_getgroups(args: &SyscallArgs) -> Result<usize, SysError> {
    let size = args.get_int(0) as usize;
    let list_addr = args.get_addr(1);
    let (_proc, data) = current_proc_and_data_mut();
    if list_addr.as_usize() == 0 {
        return Ok(data.ngroups);
    }
    if size < data.ngroups {
        err!(SysError::InvalidArgument)
    }
    for i in 0..data.ngroups {
        let addr = VA::new(list_addr.as_usize() + i * 4);
        let buf = data.groups[i].to_ne_bytes();
        data.pagetable_mut()
            .copy_to(&buf, addr)
            .map_err(|_| SysError::BadAddress)?;
    }
    Ok(data.ngroups)
}

pub fn sys_initgroups(args: &SyscallArgs) -> Result<usize, SysError> {
    let _name = try_log!(args.fetch_string(args.get_addr(0), 256));
    let group = args.get_int(1) as u32;
    let (_proc, data) = current_proc_and_data_mut();
    data.groups[0] = group;
    data.ngroups = 1;
    Ok(0)
}

pub fn sys_confstr(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = args.get_int(0) as usize;
    let buf_addr = args.get_addr(1);
    let buf_len = args.get_int(2) as usize;
    match name {
        1 => {
            let val = b"/bin:/usr/bin\0";
            let len = val.len().min(buf_len);
            if buf_len > 0 && buf_addr.as_usize() != 0 {
                let (_proc, data) = current_proc_and_data_mut();
                data.pagetable_mut()
                    .copy_to(&val[..len], buf_addr)
                    .map_err(|_| SysError::BadAddress)?;
            }
            Ok(val.len())
        }
        _ => err!(SysError::InvalidArgument),
    }
}

pub fn sys_sigaction(args: &SyscallArgs) -> Result<usize, SysError> {
    let sig = args.get_int(0) as usize;
    let act_addr = args.get_addr(1);
    let oldact_addr = args.get_addr(2);

    if sig == 0 || sig > signal::SIGNAL_MAX {
        err!(SysError::InvalidArgument)
    }
    if sig == signal::SIGKILL || sig == signal::SIGSTOP {
        err!(SysError::InvalidArgument)
    }

    let (_proc, data) = current_proc_and_data_mut();
    let idx = sig - 1;

    if oldact_addr.as_usize() != 0 {
        let old = data.sigactions.as_ref().unwrap().lock()[idx];
        let mut old_bytes = [0u8; 16];
        old_bytes[..8].copy_from_slice(&old.handler.to_ne_bytes());
        old_bytes[8..12].copy_from_slice(&old.flags.to_ne_bytes());
        old_bytes[12..].copy_from_slice(&old.mask.to_ne_bytes());
        data.pagetable_mut()
            .copy_to(&old_bytes, oldact_addr)
            .map_err(|_| SysError::BadAddress)?;
    }

    if act_addr.as_usize() != 0 {
        let mut buf = [0u8; 16];
        data.pagetable_mut()
            .copy_from(act_addr, &mut buf)
            .map_err(|_| SysError::BadAddress)?;
        let handler = usize::from_ne_bytes(buf[..8].try_into().unwrap());
        let flags = u32::from_ne_bytes(buf[8..12].try_into().unwrap());
        let mask = u32::from_ne_bytes(buf[12..16].try_into().unwrap());
        data.sigactions.as_ref().unwrap().lock()[idx] = signal::SigAction { handler, flags, mask };
    }

    Ok(0)
}

pub fn sys_sigprocmask(args: &SyscallArgs) -> Result<usize, SysError> {
    let how = args.get_int(0) as i32;
    let set_addr = args.get_addr(1);
    let oldset_addr = args.get_addr(2);

    let (_proc, data) = current_proc_and_data_mut();

    if oldset_addr.as_usize() != 0 {
        let old = data.signals.get_blocked();
        let buf = old.to_ne_bytes();
        data.pagetable_mut()
            .copy_to(&buf, oldset_addr)
            .map_err(|_| SysError::BadAddress)?;
    }

    if set_addr.as_usize() != 0 {
        let mut buf = [0u8; 4];
        data.pagetable_mut()
            .copy_from(set_addr, &mut buf)
            .map_err(|_| SysError::BadAddress)?;
        let set = u32::from_ne_bytes(buf) as usize;

        let current = data.signals.get_blocked();
        let new = match how {
            signal::SIG_BLOCK => current | (set & !((1 << 8) | (1 << 18))),
            signal::SIG_UNBLOCK => current & !set,
            signal::SIG_SETMASK => set & !((1 << 8) | (1 << 18)),
            _ => err!(SysError::InvalidArgument),
        };
        data.signals
            .blocked
            .store(new, core::sync::atomic::Ordering::Relaxed);
    }

    Ok(0)
}

pub fn sys_sigpending(args: &SyscallArgs) -> Result<usize, SysError> {
    let set_addr = args.get_addr(0);
    let (_proc, data) = current_proc_and_data_mut();
    let pending = data.signals.get_pending() as u32;
    let buf = pending.to_ne_bytes();
    data.pagetable_mut()
        .copy_to(&buf, set_addr)
        .map_err(|_| SysError::BadAddress)?;
    Ok(0)
}

pub fn sys_sigsuspend(args: &SyscallArgs) -> Result<usize, SysError> {
    let mask_addr = args.get_addr(0);
    let (_proc, data) = current_proc_and_data_mut();

    let mut buf = [0u8; 4];
    data.pagetable_mut()
        .copy_from(mask_addr, &mut buf)
        .map_err(|_| SysError::BadAddress)?;
    let new_mask = u32::from_ne_bytes(buf) as usize;

    data.signals
        .blocked
        .store(new_mask, core::sync::atomic::Ordering::Relaxed);
    let _ = data;
    let _ = _proc;

    loop {
        let proc = current_proc();
        if proc.is_killed() {
            return err!(SysError::Interrupted);
        }
        let data = unsafe { proc.data_mut() };
        let pending = data.signals.get_pending();
        let blocked = data.signals.get_blocked();
        if pending & !blocked != 0 {
            return err!(SysError::Interrupted);
        }
        let _ = data;
        proc::r#yield();
    }
}

pub fn sys_sigreturn(args: &SyscallArgs) -> Result<usize, SysError> {
    let frame_addr = args.get_addr(0);
    let (_proc, data) = current_proc_and_data_mut();

    let mut frame_buf = [0u8; core::mem::size_of::<signal::SigFrame>()];
    data.pagetable_mut()
        .copy_from(frame_addr, &mut frame_buf)
        .map_err(|_| SysError::BadAddress)?;

    let frame: &signal::SigFrame =
        unsafe { &*(frame_buf.as_ptr() as *const signal::SigFrame) };

    let tf = data.trapframe_mut();
    tf.epc = frame.epc as usize;
    tf.ra = frame.ra as usize;
    tf.sp = frame.sp as usize;
    tf.gp = frame.gp as usize;
    tf.tp = frame.tp as usize;
    tf.t0 = frame.t0 as usize;
    tf.t1 = frame.t1 as usize;
    tf.t2 = frame.t2 as usize;
    tf.s0 = frame.s0 as usize;
    tf.s1 = frame.s1 as usize;
    tf.a0 = frame.a0 as usize;
    tf.a1 = frame.a1 as usize;
    tf.a2 = frame.a2 as usize;
    tf.a3 = frame.a3 as usize;
    tf.a4 = frame.a4 as usize;
    tf.a5 = frame.a5 as usize;
    tf.a6 = frame.a6 as usize;
    tf.a7 = frame.a7 as usize;
    tf.s2 = frame.s2 as usize;
    tf.s3 = frame.s3 as usize;
    tf.s4 = frame.s4 as usize;
    tf.s5 = frame.s5 as usize;
    tf.s6 = frame.s6 as usize;
    tf.s7 = frame.s7 as usize;
    tf.s8 = frame.s8 as usize;
    tf.s9 = frame.s9 as usize;
    tf.s10 = frame.s10 as usize;
    tf.s11 = frame.s11 as usize;
    tf.t3 = frame.t3 as usize;
    tf.t4 = frame.t4 as usize;
    tf.t5 = frame.t5 as usize;
    tf.t6 = frame.t6 as usize;

    data.signals
        .blocked
        .store(frame.oldmask as usize, core::sync::atomic::Ordering::Relaxed);
    data.signals
        .in_handler
        .store(false, core::sync::atomic::Ordering::Relaxed);

    Ok(frame.a0 as usize)
}

pub fn sys_killpg(args: &SyscallArgs) -> Result<usize, SysError> {
    let pgrp = args.get_int(0) as usize;
    let sig = args.get_int(1) as usize;

    if sig > signal::SIGNAL_MAX {
        err!(SysError::InvalidArgument)
    }

    let mut found = false;
    for p in crate::proc::PROC_TABLE.iter() {
        let inner = p.inner.lock();
        if inner.state != crate::proc::ProcState::Unused {
            if *p.data().pgrp == pgrp {
                let target_pid = *inner.pid;
                drop(inner);
                p.data().signals.send_signal(sig);
                crate::signalfd::signalfd_notify(target_pid, sig);
                found = true;
            } else {
                drop(inner);
            }
        } else {
            drop(inner);
        }
    }

    if found {
        Ok(0)
    } else {
        err!(SysError::NoProcess)
    }
}

fn env_find(env: &[String], name: &str) -> Option<usize> {
    let prefix = [name, "="].concat();
    let prefix_bytes = prefix.as_bytes();
    env.iter().position(|s| s.as_bytes().starts_with(prefix_bytes))
}

pub fn sys_getenv(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = try_log!(args.fetch_string(args.get_addr(0), 256));
    let buf_addr = args.get_addr(1);
    let buf_len = args.get_int(2) as usize;

    let (_proc, data) = current_proc_and_data_mut();
    let value_bytes = {
        let idx = env_find(&data.env, &name);
        let mut buf = Vec::new();
        if let Some(idx) = idx {
            if let Some(entry) = data.env.get(idx) {
                if let Some(eq_pos) = entry.find('=') {
                    buf.extend_from_slice(entry[eq_pos + 1..].as_bytes());
                }
            }
        }
        buf
    };
    if !value_bytes.is_empty() {
        let len = value_bytes.len().min(buf_len);
        if len > 0 {
            data.pagetable_mut()
                .copy_to(&value_bytes[..len], buf_addr)
                .map_err(|_| SysError::BadAddress)?;
        }
        return Ok(len);
    }
    err!(SysError::NoProcess)
}

pub fn sys_setenv(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = try_log!(args.fetch_string(args.get_addr(0), 256));
    let value = try_log!(args.fetch_string(args.get_addr(1), 256));
    let overwrite = args.get_int(2);

    if name.is_empty() || name.contains('=') {
        err!(SysError::InvalidArgument)
    }

    let (_proc, data) = current_proc_and_data_mut();
    let entry = [name.as_str(), "=", value.as_str()].concat();

    if let Some(idx) = env_find(&data.env, &name) {
        if overwrite != 0 {
            data.env[idx] = entry;
        }
    } else {
        data.env.push(entry);
    }
    Ok(0)
}

pub fn sys_unsetenv(args: &SyscallArgs) -> Result<usize, SysError> {
    let name = try_log!(args.fetch_string(args.get_addr(0), 256));

    if name.is_empty() || name.contains('=') {
        err!(SysError::InvalidArgument)
    }

    let (_proc, data) = current_proc_and_data_mut();
    if let Some(idx) = env_find(&data.env, &name) {
        data.env.remove(idx);
    }
    Ok(0)
}

pub fn sys_clearenv(_args: &SyscallArgs) -> Result<usize, SysError> {
    let (_proc, data) = current_proc_and_data_mut();
    data.env.clear();
    Ok(0)
}

pub fn sys_getpagesize(_args: &SyscallArgs) -> Result<usize, SysError> {
    Ok(crate::riscv::PGSIZE)
}

/// Clone a process (create a thread).
///
/// Arguments (Linux clone convention):
///   a0 = flags (CLONE_VM=0x100, CLONE_THREAD=0x10000, CLONE_SETTLS=0x800)
///   a1 = child stack (0 = use parent's sp)
///   a2 = ptid (not used)
///   a3 = tls (thread pointer for CLONE_SETTLS)
///   a4 = ctid (not used)
pub fn sys_clone(args: &SyscallArgs) -> Result<usize, SysError> {
    let flags = args.get_raw(0);
    let stack = args.get_addr(1);
    let tls = args.get_raw(3); // CLONE_SETTLS: thread-local storage pointer

    match log!(proc::clone_proc(flags, stack.as_usize(), tls)) {
        Ok(pid) => Ok(*pid),
        Err(_) => Err(SysError::ResourceUnavailable),
    }
}

/// Get thread group ID (same as PID for the group leader).
pub fn sys_gettid(args: &SyscallArgs) -> Result<usize, SysError> {
    let tgid = args.proc().inner.lock().tgid;
    Ok(*tgid)
}

/// Exit all threads in the current thread group.
pub fn sys_exit_group(args: &SyscallArgs) -> ! {
    let status = args.get_int(0);
    proc::exit_group(status);
}

/// Minimal futex: FUTEX_WAIT (0) and FUTEX_WAKE (1).
///
pub fn sys_eventfd2(args: &SyscallArgs) -> Result<usize, SysError> {
    let initval = args.get_int(0) as u32;
    let flags = args.get_int(1) as u32;

    let eventfd_id = try_log!(crate::eventfd::alloc_eventfd_id(initval, flags));

    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));

    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::EventFd { eventfd_id };
    inner.readable = true;
    inner.writeable = true;
    inner.offset = 0;

    Ok(fd)
}

pub fn sys_timerfd_create(args: &SyscallArgs) -> Result<usize, SysError> {
    let clockid = args.get_int(0) as i32;
    let flags = args.get_int(1) as u32;

    let timerfd_id = try_log!(crate::timerfd::alloc_timerfd_id(clockid, flags));

    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));

    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::TimerFd { timerfd_id };
    inner.readable = true;
    inner.writeable = true;
    inner.offset = 0;

    Ok(fd)
}

pub fn sys_timerfd_settime(args: &SyscallArgs) -> Result<usize, SysError> {
    let flags = args.get_int(1) as u32;
    let new_val_addr = args.get_addr(2);
    let old_val_addr = args.get_addr(3);

    let (_, file) = try_log!(args.get_file(0));

    let timerfd_id = {
        let inner = crate::file::FILE_TABLE.inner[file.id].lock();
        match &inner.r#type {
            crate::file::FileType::TimerFd { timerfd_id } => *timerfd_id,
            _ => err!(SysError::BadDescriptor),
        }
    };

    let mut new_val = crate::timerfd::Itimerspec::default();
    let bytes = unsafe {
        core::slice::from_raw_parts_mut(
            &mut new_val as *mut _ as *mut u8,
            core::mem::size_of::<crate::timerfd::Itimerspec>(),
        )
    };
    let (_proc, data) = proc::current_proc_and_data_mut();
    let pt = data.pagetable_mut();
    if pt.copy_from(new_val_addr, bytes).is_err() {
        err!(SysError::BadAddress);
    }

    let mut old_val = crate::timerfd::Itimerspec::default();
    let ret = log!(crate::timerfd::timerfd_settime(timerfd_id, flags, &new_val, &mut old_val));

    if old_val_addr.as_usize() != 0 && ret.is_ok() {
        let old_bytes = unsafe {
            core::slice::from_raw_parts(
                &old_val as *const _ as *const u8,
                core::mem::size_of::<crate::timerfd::Itimerspec>(),
            )
        };
        let _ = pt.copy_to(old_bytes, old_val_addr);
    }

    ret.map(|_| 0)
}

pub fn sys_timerfd_gettime(args: &SyscallArgs) -> Result<usize, SysError> {
    let curr_addr = args.get_addr(1);

    let (_, file) = try_log!(args.get_file(0));

    let timerfd_id = {
        let inner = crate::file::FILE_TABLE.inner[file.id].lock();
        match &inner.r#type {
            crate::file::FileType::TimerFd { timerfd_id } => *timerfd_id,
            _ => err!(SysError::BadDescriptor),
        }
    };

    let mut curr = crate::timerfd::Itimerspec::default();
    log!(crate::timerfd::timerfd_gettime(timerfd_id, &mut curr))?;

    let bytes = unsafe {
        core::slice::from_raw_parts(
            &curr as *const _ as *const u8,
            core::mem::size_of::<crate::timerfd::Itimerspec>(),
        )
    };
    let (_proc, data) = proc::current_proc_and_data_mut();
    let pt = data.pagetable_mut();
    if pt.copy_to(bytes, curr_addr).is_err() {
        err!(SysError::BadAddress);
    }

    Ok(0)
}

pub fn sys_memfd_create(args: &SyscallArgs) -> Result<usize, SysError> {
    let _name_addr = args.get_addr(0);
    let flags = args.get_int(1) as u32;
    let memfd_id = try_log!(crate::memfd::alloc_memfd_id());
    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));
    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::MemFd { memfd_id };
    inner.readable = true;
    inner.writeable = true;
    inner.offset = 0;
    if (flags & crate::memfd::MFD_CLOEXEC) != 0 {
        inner.nonblocking = false;
    }
    Ok(fd)
}

pub fn sys_pidfd_open(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0) as usize;
    let _flags = args.get_int(1) as u32;
    if !crate::proc::is_valid_pid(pid) {
        err!(SysError::NoEntry);
    }
    let pidfd_id = try_log!(crate::pidfd::alloc_pidfd_id(pid));
    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));
    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::PidFd { pidfd_id };
    inner.readable = true;
    inner.writeable = false;
    Ok(fd)
}

/// a0 = uaddr (user address)
/// a1 = futex_op (WAIT=0, WAKE=1)
/// a2 = val    (expected value for WAIT, wake count for WAKE)
pub fn sys_futex(args: &SyscallArgs) -> Result<usize, SysError> {
    let addr = args.get_addr(0);
    let op = args.get_int(1) as i32;
    let val = args.get_int(2) as u32;

    match op & 0xf {
        0 => {
            // FUTEX_WAIT: sleep if *addr == val, else EAGAIN
            let (proc, mut data) = current_proc_and_data_mut();
            let mut buf = [0u8; 4];
            if data.pagetable_mut().copy_from(addr, &mut buf).is_err() {
                return Err(SysError::BadAddress);
            }
            let curr = u32::from_le_bytes(buf);
            if curr != val {
                return Err(SysError::ResourceUnavailable); // EAGAIN
            }
            let _ = data;
            let mut inner = proc.inner.lock();
            inner.channel = Some(Channel::Address(addr.as_usize()));
            inner.state = ProcState::Sleeping;
            let context = unsafe { &mut proc.data_mut().context };
            inner = proc::sched(inner, context);
            inner.channel = None;
            Ok(0)
        }
        1 => {
            // FUTEX_WAKE: wake up to `val` waiters
            let channel = Channel::Address(addr.as_usize());
            let woken = wakeup_n(channel, val as usize);
            Ok(woken)
        }
        _ => Err(SysError::NotImplemented),
    }
}

// ─── v4.4 stubs (to be implemented in subsequent versions) ───

pub fn sys_getrandom(args: &SyscallArgs) -> Result<usize, SysError> {
    let buf = args.get_addr(0);
    let len = args.get_int(1) as usize;
    let _flags = args.get_int(2) as u32;

    if len == 0 {
        return Ok(0);
    }

    let mut bytes = alloc::vec![0u8; len];
    crate::rng::rand_bytes(&mut bytes);
    let (_proc, data) = current_proc_and_data_mut();
    if log!(data.pagetable_mut().copy_to(&bytes, buf)).is_err() {
        err!(SysError::BadAddress);
    }
    Ok(len)
}

pub fn sys_close_range(args: &SyscallArgs) -> Result<usize, SysError> {
    let first = args.get_int(0) as usize;
    let last = args.get_int(1) as usize;
    let _flags = args.get_int(2) as u32;

    let last = last.min(crate::param::NOFILE - 1);
    for fd in first..=last {
        let data = proc::current_proc().data();
        let mut files = data.open_files.as_ref().unwrap().files.lock();
        if let Some(mut file) = files[fd].take() {
            drop(files);
            file.close();
        }
    }
    Ok(0)
}

pub fn sys_prctl(_args: &SyscallArgs) -> Result<usize, SysError> {
    Err(SysError::NotImplemented)
}

pub fn sys_inotify_init1(args: &SyscallArgs) -> Result<usize, SysError> {
    let _flags = args.get_int(0) as u32;
    let inotify_id = try_log!(crate::inotify::alloc_inotify_id());
    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));
    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::Inotify { inotify_id };
    inner.readable = true;
    inner.writeable = false;
    Ok(fd)
}

pub fn sys_inotify_add_watch(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, file) = try_log!(args.get_file(0));
    let path_addr = args.get_addr(1);
    let mask = args.get_int(2) as u32;
    let path_str = try_log!(args.fetch_string(path_addr, crate::param::MAXPATH));
    let path = crate::fs::Path::new(&path_str);
    let inode = try_log!(log!(path.resolve()));
    let inotify_id = {
        let inner = crate::file::FILE_TABLE.inner[file.id].lock();
        match &inner.r#type {
            crate::file::FileType::Inotify { inotify_id } => *inotify_id,
            _ => err!(SysError::BadDescriptor),
        }
    };
    let wd = try_log!(crate::inotify::inotify_add_watch(inotify_id, inode.dev, inode.inum, mask));
    Ok(wd as usize)
}

pub fn sys_inotify_rm_watch(args: &SyscallArgs) -> Result<usize, SysError> {
    let (_, file) = try_log!(args.get_file(0));
    let wd = args.get_int(1) as i32;
    let inotify_id = {
        let inner = crate::file::FILE_TABLE.inner[file.id].lock();
        match &inner.r#type {
            crate::file::FileType::Inotify { inotify_id } => *inotify_id,
            _ => err!(SysError::BadDescriptor),
        }
    };
    try_log!(crate::inotify::inotify_rm_watch(inotify_id, wd));
    Ok(0)
}

pub fn sys_signalfd4(args: &SyscallArgs) -> Result<usize, SysError> {
    let fd_hint = args.get_int(0) as usize;
    let mask_addr = args.get_addr(1);
    let sizemask = args.get_int(2) as usize;
    let flags = args.get_int(3) as u32;

    let mut mask = 0u32;
    let (_proc, data) = current_proc_and_data_mut();
    let copy_size = sizemask.min(8);
    if copy_size > 0 {
        let mut buf = [0u8; 8];
        data.pagetable_mut()
            .copy_from(mask_addr, &mut buf[..copy_size])
            .map_err(|_| SysError::BadAddress)?;
        mask = u32::from_ne_bytes(buf[..4].try_into().unwrap());
    }

    let pid = *_proc.inner.lock().pid;
    let _ = fd_hint;
    let signalfd_id = try_log!(crate::signalfd::alloc_signalfd_id(pid, mask));

    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));

    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::Signalfd { signalfd_id };
    inner.readable = true;
    inner.writeable = false;
    inner.nonblocking = (flags & crate::signalfd::SFD_NONBLOCK) != 0;

    Ok(fd)
}

pub fn sys_setns(args: &SyscallArgs) -> Result<usize, SysError> {
    let _nstype = args.get_int(1) as u32;
    let (_fd_num, file) = try_log!(args.get_file(0));
    let inner = crate::file::FILE_TABLE.inner[file.id].lock();
    match &inner.r#type {
        crate::file::FileType::NsFd { ns_proxy, nstype } => {
            let (_, data) = current_proc_and_data_mut();
            let current = data.ns.as_ref().unwrap();
            let new_ns = current.clone_with_override(*nstype, ns_proxy);
            data.ns = Some(new_ns);
            Ok(0)
        }
        _ => Err(SysError::BadDescriptor),
    }
}

pub fn sys_nsopen(args: &SyscallArgs) -> Result<usize, SysError> {
    let pid = args.get_int(0) as usize;
    let nstype_val = args.get_int(1) as u32;
    if !crate::proc::is_valid_pid(pid) {
        return Err(SysError::NoEntry);
    }
    let nstype = match nstype_val {
        0 => crate::namespace::NsType::Mount,
        1 => crate::namespace::NsType::Cgroup,
        2 => crate::namespace::NsType::Uts,
        3 => crate::namespace::NsType::Ipc,
        4 => crate::namespace::NsType::User,
        5 => crate::namespace::NsType::Pid,
        6 => crate::namespace::NsType::Net,
        _ => return Err(SysError::InvalidArgument),
    };
    let ns_proxy = {
        let mut found: Option<alloc::sync::Arc<crate::namespace::NsProxy>> = None;
        for p in crate::proc::PROC_TABLE.iter() {
            if *p.inner.lock().pid == pid {
                let data = p.data();
                found = data.ns.clone().map(alloc::sync::Arc::new);
                break;
            }
        }
        found.ok_or(SysError::NoEntry)?
    };
    let file = try_log!(crate::file::File::alloc());
    let fd = try_log!(crate::sysfile::fd_alloc(file.clone()));
    let mut inner = crate::file::FILE_TABLE.inner[file.id].lock();
    inner.r#type = crate::file::FileType::NsFd { ns_proxy, nstype };
    inner.readable = true;
    inner.writeable = false;
    Ok(fd)
}

pub fn sys_unshare(args: &SyscallArgs) -> Result<usize, SysError> {
    let flags = args.get_int(0);
    let (_, data) = current_proc_and_data_mut();
    let parent_ns = data.ns.as_ref().unwrap();
    data.ns = Some(crate::namespace::NsProxy::from_parent(parent_ns, flags as usize));
    Ok(0)
}

pub fn sys_sethostname(args: &SyscallArgs) -> Result<usize, SysError> {
    let hostname_addr = args.get_addr(0);
    let len = args.get_int(1) as usize;
    let (proc, _data) = current_proc_and_data_mut();
    // copy hostname from user space
    let mut buf = alloc::vec![0u8; len];
    let src = hostname_addr;
    let dst = &mut buf;
    try_log!(proc::copy_from_user(src, dst).map_err(|_| SysError::BadAddress));
    let (_, data) = current_proc_and_data_mut();
    let ns = data.ns.as_ref().unwrap();
    let mut uts_data = ns.uts.data.lock();
    uts_data.set_hostname(dst).map_err(|_| SysError::InvalidArgument)?;
    Ok(0)
}

pub fn sys_gethostname(args: &SyscallArgs) -> Result<usize, SysError> {
    let buf_addr = args.get_addr(0);
    let len = args.get_int(1) as usize;
    let (_, data) = current_proc_and_data_mut();
    let ns = data.ns.as_ref().unwrap();
    let uts_data = ns.uts.data.lock();
    let hostname = uts_data.hostname();
    if hostname.len() > len {
        return Err(SysError::InvalidArgument);
    }
    let src = hostname;
    let dst = buf_addr;
    try_log!(proc::copy_to_user(src, dst).map_err(|_| SysError::BadAddress));
    Ok(hostname.len())
}

pub fn sys_capget(args: &SyscallArgs) -> Result<usize, SysError> {
    let hdr_addr = args.get_addr(0);
    let data_addr = args.get_addr(1);
    crate::capability::sys_capget(hdr_addr, data_addr)
}

pub fn sys_capset(args: &SyscallArgs) -> Result<usize, SysError> {
    let hdr_addr = args.get_addr(0);
    let data_addr = args.get_addr(1);
    crate::capability::sys_capset(hdr_addr, data_addr)
}

pub fn sys_seccomp(args: &SyscallArgs) -> Result<usize, SysError> {
    let op = args.get_int(0) as usize;
    let flags = args.get_int(1) as usize;
    let args_addr = args.get_addr(2);
    crate::seccomp::sys_seccomp(op, flags, args_addr)
}

pub fn sys_overlay_mount(args: &SyscallArgs) -> Result<usize, SysError> {
    let mp = args.fetch_string(args.get_addr(0), 256)?;
    let up = args.fetch_string(args.get_addr(1), 256)?;
    let lo = args.fetch_string(args.get_addr(2), 256)?;
    crate::overlay::sys_mount(&mp, &up, &lo)
}

pub fn sys_overlay_umount(args: &SyscallArgs) -> Result<usize, SysError> {
    let mp = args.fetch_string(args.get_addr(0), 256)?;
    crate::overlay::sys_umount(&mp)
}

pub fn sys_pivot_root(args: &SyscallArgs) -> Result<usize, SysError> {
    let new_root_str = args.fetch_string(args.get_addr(0), 256)?;
    let put_old_str = args.fetch_string(args.get_addr(1), 256)?;

    // Resolve new_root (must be a directory)
    let new_root_inode = Path::new(&new_root_str)
        .resolve()
        .map_err(|_| SysError::InvalidArgument)?;

    let new_root_inum = new_root_inode.inum;
    {
        let inner = new_root_inode.lock();
        if inner.r#type != InodeType::Directory {
            new_root_inode.unlock_put(inner);
            return Err(SysError::InvalidArgument);
        }
    }

    // Resolve put_old (must be a directory under new_root)
    let put_old_inode = Path::new(&put_old_str)
        .resolve()
        .map_err(|_| SysError::InvalidArgument)?;

    {
        let inner = put_old_inode.lock();
        if inner.r#type != InodeType::Directory {
            put_old_inode.unlock_put(inner);
            new_root_inode.put();
            return Err(SysError::InvalidArgument);
        }
    }

    put_old_inode.put();

    // Set process root to new_root
    let (_proc, mut data) = current_proc_and_data_mut();
    if let Some(old_root) = data.root.take() {
        old_root.put();
    }
    data.root = Some(new_root_inode);

    Ok(0)
}
