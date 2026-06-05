use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::memlayout::QEMU_POWER;
use crate::param::MMAP_BASE;
use crate::proc::{self, Channel, Pid, current_proc, current_proc_and_data_mut};
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
    let pid = args.proc().inner.lock().pid;
    Ok(*pid)
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
        let old = data.sigactions[idx];
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
        data.sigactions[idx] = signal::SigAction { handler, flags, mask };
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
                drop(inner);
                p.data().signals.send_signal(sig);
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