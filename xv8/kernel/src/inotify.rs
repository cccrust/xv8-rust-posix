use alloc::vec;
use alloc::vec::Vec;

use crate::param::NINOTIFY;
use crate::proc::Channel;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub const IN_ACCESS: u32 = 0x00000001;
pub const IN_MODIFY: u32 = 0x00000002;
pub const IN_ATTRIB: u32 = 0x00000004;
pub const IN_CLOSE_WRITE: u32 = 0x00000008;
pub const IN_CLOSE_NOWRITE: u32 = 0x00000010;
pub const IN_OPEN: u32 = 0x00000020;
pub const IN_MOVED_FROM: u32 = 0x00000040;
pub const IN_MOVED_TO: u32 = 0x00000080;
pub const IN_CREATE: u32 = 0x00000100;
pub const IN_DELETE: u32 = 0x00000200;
pub const IN_DELETE_SELF: u32 = 0x00000400;
pub const IN_MOVE_SELF: u32 = 0x00000800;
pub const IN_ALL_EVENTS: u32 = 0x00000fff;
pub const IN_ONLYDIR: u32 = 0x01000000;
pub const IN_NONBLOCK: u32 = 0x00004000;
pub const IN_ISDIR: u32 = 0x40000000;

#[repr(C)]
pub struct InotifyEvent {
    pub wd: i32,
    pub mask: u32,
    pub cookie: u32,
    pub len: u32,
}

pub struct InotifyWatch {
    pub wd: i32,
    pub dev: u32,
    pub inum: u32,
    pub mask: u32,
}

pub struct InotifyInstance {
    pub events: Vec<u8>,
    pub watches: Vec<InotifyWatch>,
    pub waiting: bool,
    pub next_wd: i32,
}

pub struct InotifyTable {
    pub instances: [Option<InotifyInstance>; NINOTIFY],
}

static INOTIFY_TABLE: SpinLock<InotifyTable> = SpinLock::new(
    InotifyTable {
        instances: [const { None }; NINOTIFY],
    },
    "inotify_table",
);

pub fn alloc_inotify_id() -> Result<usize, SysError> {
    let mut table = INOTIFY_TABLE.lock();
    for i in 0..table.instances.len() {
        if table.instances[i].is_none() {
            table.instances[i] = Some(InotifyInstance {
                events: Vec::new(),
                watches: Vec::new(),
                waiting: false,
                next_wd: 1,
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_inotify_id(id: usize) {
    let mut table = INOTIFY_TABLE.lock();
    if id < table.instances.len() {
        table.instances[id] = None;
    }
}

pub fn inotify_add_watch(id: usize, dev: u32, inum: u32, mask: u32) -> Result<i32, SysError> {
    let mut table = INOTIFY_TABLE.lock();
    let Some(ref mut inst) = table.instances[id] else {
        err!(SysError::BadDescriptor)
    };

    for watch in &inst.watches {
        if watch.dev == dev && watch.inum == inum {
            let wd = watch.wd;
            let idx = inst.watches.iter().position(|w| w.wd == wd).unwrap();
            inst.watches[idx].mask = mask;
            return Ok(wd);
        }
    }

    let wd = inst.next_wd;
    inst.next_wd += 1;
    inst.watches.push(InotifyWatch { wd, dev, inum, mask });
    Ok(wd)
}

pub fn inotify_rm_watch(id: usize, wd: i32) -> Result<(), SysError> {
    let mut table = INOTIFY_TABLE.lock();
    let Some(ref mut inst) = table.instances[id] else {
        err!(SysError::BadDescriptor)
    };
    let pos = inst.watches.iter().position(|w| w.wd == wd).ok_or(SysError::NoEntry)?;
    inst.watches.remove(pos);
    Ok(())
}

pub fn inotify_read(id: usize, buf: &mut [u8]) -> Result<usize, SysError> {
    let mut table = INOTIFY_TABLE.lock();
    let Some(ref mut inst) = table.instances[id] else {
        err!(SysError::BadDescriptor)
    };

    if inst.events.is_empty() {
        inst.waiting = true;
        err!(SysError::ResourceUnavailable)
    }

    let n = inst.events.len().min(buf.len());
    buf[..n].copy_from_slice(&inst.events[..n]);
    let remaining = inst.events.len() - n;
    if remaining > 0 {
        let tail = inst.events.split_off(n);
        inst.events = tail;
    } else {
        inst.events.clear();
    }
    inst.waiting = false;
    Ok(n)
}

pub fn inotify_readiness(id: usize) -> (bool, bool) {
    let table = INOTIFY_TABLE.lock();
    if let Some(ref inst) = table.instances[id] {
        (!inst.events.is_empty(), false)
    } else {
        (false, false)
    }
}

pub fn notify(dev: u32, inum: u32, mask: u32, cookie: u32, name: &str) {
    let mut table = INOTIFY_TABLE.lock();
    let mut to_wake: Vec<usize> = Vec::new();

    for (inst_id, slot) in table.instances.iter_mut().enumerate() {
        let Some(inst) = slot else { continue };
        let mut matched = false;

        for watch in &inst.watches {
            if watch.dev == dev && watch.inum == inum && (watch.mask & mask) != 0 {
                let name_bytes = name.as_bytes();
                let event_hdr = InotifyEvent {
                    wd: watch.wd,
                    mask,
                    cookie,
                    len: name_bytes.len() as u32,
                };
                let hdr_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &event_hdr as *const _ as *const u8,
                        core::mem::size_of::<InotifyEvent>(),
                    )
                };
                inst.events.extend_from_slice(hdr_bytes);
                inst.events.extend_from_slice(name_bytes);
                let total = core::mem::size_of::<InotifyEvent>() + name_bytes.len();
                let padded = (total + 3) & !3;
                while inst.events.len() < padded {
                    inst.events.push(0);
                }
                matched = true;
            }
        }

        if matched && inst.waiting {
            to_wake.push(inst_id);
        }
    }

    drop(table);
    for inst_id in to_wake {
        crate::proc::wakeup(Channel::Inotify(inst_id));
    }
}
