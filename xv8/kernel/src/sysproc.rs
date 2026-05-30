use alloc::vec;

use crate::memlayout::QEMU_POWER;
use crate::proc::{self, Channel, Pid, current_proc};
use crate::rng::rand_bytes;
use crate::syscall::{SysError, SyscallArgs};
use crate::trap::TICKS;

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

    // Safety: kernel will return an error if the process does not exist.
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

    let tv = TimeVal {
        sec: (*TICKS.lock() / 100) as u32,
        usec: 0,
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
