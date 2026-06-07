use crate::param::NPIDFD;
use crate::proc;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub struct PidFdState {
    pub pid: usize,
}

pub struct PidFdTable {
    pub entries: [Option<PidFdState>; NPIDFD],
    pub next_id: usize,
}

static PIDFD_TABLE: SpinLock<PidFdTable> = SpinLock::new(
    PidFdTable {
        entries: [const { None }; NPIDFD],
        next_id: 0,
    },
    "pidfd_table",
);

pub fn alloc_pidfd_id(pid: usize) -> Result<usize, SysError> {
    let mut table = PIDFD_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            table.entries[i] = Some(PidFdState { pid });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_pidfd_id(id: usize) {
    let mut table = PIDFD_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

pub fn pidfd_is_alive(id: usize) -> bool {
    let table = PIDFD_TABLE.lock();
    match &table.entries[id] {
        Some(state) => {
            let pids = proc::all_pids();
            pids.contains(&state.pid)
        }
        None => false,
    }
}

pub fn pidfd_get_pid(id: usize) -> Option<usize> {
    let table = PIDFD_TABLE.lock();
    table.entries[id].as_ref().map(|s| s.pid)
}
