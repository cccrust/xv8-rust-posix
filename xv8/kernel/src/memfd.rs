use alloc::vec::Vec;
use crate::param::NMEMFD;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub const MFD_CLOEXEC: u32 = 0x0001;

pub struct MemFdState {
    pub buf: Vec<u8>,
}

pub struct MemFdTable {
    pub entries: [Option<MemFdState>; NMEMFD],
    pub next_id: usize,
}

static MEMFD_TABLE: SpinLock<MemFdTable> = SpinLock::new(
    MemFdTable {
        entries: [const { None }; NMEMFD],
        next_id: 0,
    },
    "memfd_table",
);

pub fn alloc_memfd_id() -> Result<usize, SysError> {
    let mut table = MEMFD_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            table.entries[i] = Some(MemFdState {
                buf: Vec::new(),
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_memfd_id(id: usize) {
    let mut table = MEMFD_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

pub fn memfd_read(id: usize, offset: usize, buf: &mut [u8]) -> Result<usize, SysError> {
    let mut table = MEMFD_TABLE.lock();
    let Some(ref state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };
    if offset >= state.buf.len() {
        return Ok(0);
    }
    let n = (buf.len()).min(state.buf.len() - offset);
    buf[..n].copy_from_slice(&state.buf[offset..offset + n]);
    Ok(n)
}

pub fn memfd_write(id: usize, offset: usize, buf: &[u8]) -> Result<usize, SysError> {
    let mut table = MEMFD_TABLE.lock();
    let Some(ref mut state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };
    let end = offset + buf.len();
    if end > state.buf.len() {
        state.buf.resize(end, 0);
    }
    state.buf[offset..end].copy_from_slice(buf);
    Ok(buf.len())
}

pub fn memfd_size(id: usize) -> Result<usize, SysError> {
    let table = MEMFD_TABLE.lock();
    let Some(ref state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };
    Ok(state.buf.len())
}
