use crate::param::NEVENTFD;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub const EFD_SEMAPHORE: u32 = 0x0001;
pub const EFD_NONBLOCK: u32 = 0x0800;
pub const EFD_CLOEXEC: u32 = 0x20000;

#[derive(Debug, Clone)]
pub struct EventFdState {
    pub counter: u64,
    pub semaphore: bool,
    pub waiting: bool,
}

pub struct EventFdTable {
    pub entries: [Option<EventFdState>; NEVENTFD],
    pub next_id: usize,
}

static EVENTFD_TABLE: SpinLock<EventFdTable> = SpinLock::new(
    EventFdTable {
        entries: [const { None }; NEVENTFD],
        next_id: 0,
    },
    "eventfd_table",
);

pub fn alloc_eventfd_id(initval: u32, flags: u32) -> Result<usize, SysError> {
    let mut table = EVENTFD_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            let semaphore = (flags & EFD_SEMAPHORE) != 0;
            table.entries[i] = Some(EventFdState {
                counter: initval as u64,
                semaphore,
                waiting: false,
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_eventfd_id(id: usize) {
    let mut table = EVENTFD_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

pub fn eventfd_read(id: usize) -> Result<u64, SysError> {
    let mut table = EVENTFD_TABLE.lock();
    let Some(ref mut state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    if state.counter == 0 {
        state.waiting = true;
        err!(SysError::ResourceUnavailable)
    }

    let val = if state.semaphore {
        state.counter -= 1;
        1
    } else {
        let v = state.counter;
        state.counter = 0;
        v
    };

    state.waiting = false;
    Ok(val)
}

pub fn eventfd_write(id: usize, val: u64) -> Result<(), SysError> {
    if val == u64::MAX {
        err!(SysError::InvalidArgument);
    }

    let mut table = EVENTFD_TABLE.lock();
    let Some(ref mut state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    let was_zero = state.counter == 0;
    let new_val = state.counter.wrapping_add(val);
    if new_val < state.counter && !state.semaphore {
        err!(SysError::ResourceUnavailable);
    }
    state.counter = new_val;

    let needs_wake = was_zero && state.waiting;
    drop(table);

    if needs_wake {
        crate::proc::wakeup(crate::proc::Channel::EventFd(id));
    }

    Ok(())
}

pub fn eventfd_readiness(id: usize) -> (bool, bool) {
    let table = EVENTFD_TABLE.lock();
    match &table.entries[id] {
        Some(state) => {
            let readable = state.counter > 0;
            let writeable = true;
            (readable, writeable)
        }
        None => (false, false),
    }
}

pub fn eventfd_wait(id: usize) {
    loop {
        {
            let table = EVENTFD_TABLE.lock();
            let Some(ref state) = table.entries[id] else {
                return;
            };
            if state.counter > 0 {
                return;
            }
            _ = state;
        }
        crate::proc::sleep(
            crate::proc::Channel::EventFd(id),
            crate::proc::current_proc().inner.lock(),
        );
    }
}
