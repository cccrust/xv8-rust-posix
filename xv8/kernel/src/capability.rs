use core::fmt;

use crate::spinlock::SpinLock;
use crate::syscall::SysError;
use crate::vm::VA;

// Capability constants (bit positions, matching Linux)
pub const CAP_CHOWN: usize = 0;
pub const CAP_DAC_OVERRIDE: usize = 1;
pub const CAP_DAC_READ_SEARCH: usize = 2;
pub const CAP_FOWNER: usize = 3;
pub const CAP_FSETID: usize = 4;
pub const CAP_KILL: usize = 5;
pub const CAP_SETGID: usize = 6;
pub const CAP_SETUID: usize = 7;
pub const CAP_SETPCAP: usize = 8;
pub const CAP_NET_BIND_SERVICE: usize = 10;
pub const CAP_NET_BROADCAST: usize = 11;
pub const CAP_NET_ADMIN: usize = 12;
pub const CAP_NET_RAW: usize = 13;
pub const CAP_IPC_LOCK: usize = 14;
pub const CAP_IPC_OWNER: usize = 15;
pub const CAP_SYS_CHROOT: usize = 18;
pub const CAP_SYS_PTRACE: usize = 19;
pub const CAP_SYS_ADMIN: usize = 21;
pub const CAP_SYS_RESOURCE: usize = 24;
pub const CAP_SYS_TIME: usize = 25;
pub const CAP_SYS_TTY_CONFIG: usize = 26;
pub const CAP_LEASE: usize = 28;
pub const CAP_AUDIT_WRITE: usize = 29;
pub const CAP_AUDIT_CONTROL: usize = 30;

pub const CAP_LAST_CAP: usize = 30;
pub const CAP_MASK_LEN: usize = 2; // two u32 words

fn cap_valid(cap: usize) -> bool {
    cap <= CAP_LAST_CAP
}

#[derive(Debug, Clone, Copy)]
pub struct CapUserHeader {
    pub version: u32,
    pub pid: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct CapUserData {
    pub effective: u32,
    pub permitted: u32,
    pub inheritable: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CapUserData64 {
    pub effective: u64,
    pub permitted: u64,
    pub inheritable: u64,
}

#[derive(Debug, Clone)]
pub struct CapabilityState {
    pub effective: u64,
    pub permitted: u64,
    pub inheritable: u64,
    pub bounding: u64,
    pub ambient: u64,
}

impl CapabilityState {
    pub const fn new() -> Self {
        Self {
            effective: !0u64,
            permitted: !0u64,
            inheritable: !0u64,
            bounding: !0u64,
            ambient: !0u64,
        }
    }

    pub fn has_cap(&self, cap: usize) -> bool {
        if cap > 63 {
            return false;
        }
        (self.effective >> cap) & 1 != 0
    }

    pub fn drop_cap(&mut self, cap: usize) {
        if cap > 63 {
            return;
        }
        self.effective &= !(1u64 << cap);
        self.permitted &= !(1u64 << cap);
        self.inheritable &= !(1u64 << cap);
        self.bounding &= !(1u64 << cap);
        self.ambient &= !(1u64 << cap);
    }

    pub fn set_effective(&mut self, mask: u64) {
        self.effective = mask & self.permitted;
    }

    pub fn set_permitted(&mut self, mask: u64) {
        self.permitted = mask & self.bounding;
        self.effective &= self.permitted;
    }

    pub fn set_inheritable(&mut self, mask: u64) {
        self.inheritable = mask & self.bounding;
    }
}

pub fn sys_capget(hdr_addr: VA, data_addr: VA) -> Result<usize, SysError> {
    let proc = crate::proc::current_proc();
    let data = proc.data();

    let mut buf = [0u8; 8];
    if crate::proc::copy_from_user(hdr_addr, &mut buf).is_err() {
        return Err(SysError::BadAddress);
    }
    let _version = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    let _pid = i32::from_le_bytes(buf[4..8].try_into().unwrap()) as usize;

    let caps = data.caps.lock();
    let out = CapUserData {
        effective: caps.effective as u32,
        permitted: caps.permitted as u32,
        inheritable: caps.inheritable as u32,
    };
    let out_bytes = unsafe {
        core::slice::from_raw_parts(
            &out as *const _ as *const u8,
            core::mem::size_of::<CapUserData>(),
        )
    };
    if crate::proc::copy_to_user(out_bytes, data_addr).is_err() {
        return Err(SysError::BadAddress);
    }
    Ok(0)
}

pub fn sys_capset(hdr_addr: VA, data_addr: VA) -> Result<usize, SysError> {
    let mut buf = [0u8; 12];
    if crate::proc::copy_from_user(data_addr, &mut buf).is_err() {
        return Err(SysError::BadAddress);
    }
    let data = CapUserData {
        effective: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        permitted: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        inheritable: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
    };

    let (_proc, mut pdata) = crate::proc::current_proc_and_data_mut();
    let mut caps = pdata.caps.lock();
    caps.set_permitted(data.permitted as u64);
    caps.set_effective(data.effective as u64);
    caps.set_inheritable(data.inheritable as u64);
    Ok(0)
}
