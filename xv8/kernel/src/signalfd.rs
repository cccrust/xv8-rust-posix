use crate::param::NSIGNALFD;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub const SFD_CLOEXEC: u32 = 0x0001;
pub const SFD_NONBLOCK: u32 = 0x0004;

pub const SIGNALFD_QUEUE_DEPTH: usize = 64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignalfdSiginfo {
    pub ssi_signo: u32,
    pub ssi_errno: i32,
    pub ssi_code: i32,
    pub ssi_pid: u32,
    pub ssi_uid: u32,
    pub ssi_fd: i32,
    pub ssi_tid: u32,
    pub ssi_band: u32,
    pub ssi_overrun: u32,
    pub ssi_trapno: u32,
    pub ssi_status: i32,
    pub ssi_int: i32,
    pub ssi_ptr: u64,
    pub ssi_utime: u64,
    pub ssi_stime: u64,
    pub ssi_addr: u64,
    pub _pad: [u8; 48],
}

impl Default for SignalfdSiginfo {
    fn default() -> Self {
        Self {
            ssi_signo: 0,
            ssi_errno: 0,
            ssi_code: 0,
            ssi_pid: 0,
            ssi_uid: 0,
            ssi_fd: 0,
            ssi_tid: 0,
            ssi_band: 0,
            ssi_overrun: 0,
            ssi_trapno: 0,
            ssi_status: 0,
            ssi_int: 0,
            ssi_ptr: 0,
            ssi_utime: 0,
            ssi_stime: 0,
            ssi_addr: 0,
            _pad: [0; 48],
        }
    }
}

#[derive(Debug)]
pub struct SignalfdInstance {
    pub mask: u32,
    pub pid: usize,
    pub queue: alloc::vec::Vec<SignalfdSiginfo>,
    pub waiting: bool,
}

pub struct SignalfdTable {
    pub entries: [Option<SignalfdInstance>; NSIGNALFD],
    pub next_id: usize,
}

static SIGNALFD_TABLE: SpinLock<SignalfdTable> = SpinLock::new(
    SignalfdTable {
        entries: [const { None }; NSIGNALFD],
        next_id: 0,
    },
    "signalfd_table",
);

pub fn alloc_signalfd_id(pid: usize, mask: u32) -> Result<usize, SysError> {
    let mut table = SIGNALFD_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            table.entries[i] = Some(SignalfdInstance {
                mask,
                pid,
                queue: alloc::vec::Vec::with_capacity(SIGNALFD_QUEUE_DEPTH),
                waiting: false,
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_signalfd_id(id: usize) {
    let mut table = SIGNALFD_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

pub fn signalfd_read(id: usize, buf: &mut [u8]) -> Result<usize, SysError> {
    let mut table = SIGNALFD_TABLE.lock();
    let Some(ref mut inst) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    if inst.queue.is_empty() {
        inst.waiting = true;
        err!(SysError::ResourceUnavailable)
    }

    let entry_size = core::mem::size_of::<SignalfdSiginfo>();
    let max_entries = buf.len() / entry_size;
    let avail = inst.queue.len().min(max_entries);
    let total_bytes = avail * entry_size;

    for i in 0..avail {
        let entry = &inst.queue[i];
        let src = unsafe {
            core::slice::from_raw_parts(
                entry as *const SignalfdSiginfo as *const u8,
                entry_size,
            )
        };
        let dst = &mut buf[i * entry_size..(i + 1) * entry_size];
        dst.copy_from_slice(src);
    }

    inst.queue.drain(0..avail);
    inst.waiting = false;

    Ok(total_bytes)
}

pub fn signalfd_readiness(id: usize) -> (bool, bool) {
    let table = SIGNALFD_TABLE.lock();
    match &table.entries[id] {
        Some(inst) => {
            let readable = !inst.queue.is_empty();
            (readable, true)
        }
        None => (false, false),
    }
}

pub fn signalfd_notify(pid: usize, sig: usize) -> bool {
    if sig == 0 || sig > 32 {
        return false;
    }

    let mut table = SIGNALFD_TABLE.lock();
    let sig_bit = 1u32 << (sig - 1);
    let mut woken = false;

    for entry in table.entries.iter_mut() {
        let Some(inst) = entry.as_mut() else { continue };
        if inst.pid != pid {
            continue;
        }
        if inst.mask & sig_bit == 0 {
            continue;
        }

        if inst.queue.len() < SIGNALFD_QUEUE_DEPTH {
            let info = SignalfdSiginfo {
                ssi_signo: sig as u32,
                ssi_code: 0,
                ssi_pid: 0,
                ..Default::default()
            };
            inst.queue.push(info);
        }

        if inst.waiting {
            woken = true;
        }
    }

    drop(table);

    if woken {
        crate::proc::wakeup(crate::proc::Channel::Signalfd(pid));
    }

    true
}

pub fn any_matching(pid: usize) -> bool {
    let table = SIGNALFD_TABLE.lock();
    table.entries.iter().any(|e| {
        if let Some(inst) = e.as_ref() {
            inst.pid == pid && !inst.queue.is_empty()
        } else {
            false
        }
    })
}
