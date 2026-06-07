use crate::param::NTIMERFD;
use crate::spinlock::SpinLock;
use crate::syscall::SysError;

pub const TFD_NONBLOCK: u32 = 0x0800;
pub const TFD_CLOEXEC: u32 = 0x20000;
pub const TFD_TIMER_ABSTIME: u32 = 0x0001;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Itimerspec {
    pub it_interval: Timespec,
    pub it_value: Timespec,
}

const TICKS_PER_SEC: u64 = 100;

fn timespec_to_ticks(ts: &Timespec) -> u64 {
    if ts.tv_sec < 0 || ts.tv_nsec < 0 {
        return 0;
    }
    let total_ms = (ts.tv_sec as u64) * 1000 + (ts.tv_nsec as u64) / 1_000_000;
    total_ms / 10
}

#[derive(Debug)]
pub struct TimerFdState {
    pub clockid: i32,
    pub flags: u32,
    pub it_value_ticks: u64,
    pub it_interval_ticks: u64,
    pub expirations: u64,
    pub waiting: bool,
    pub armed: bool,
}

pub struct TimerFdTable {
    pub entries: [Option<TimerFdState>; NTIMERFD],
    pub next_id: usize,
}

static TIMERFD_TABLE: SpinLock<TimerFdTable> = SpinLock::new(
    TimerFdTable {
        entries: [const { None }; NTIMERFD],
        next_id: 0,
    },
    "timerfd_table",
);

pub fn alloc_timerfd_id(clockid: i32, flags: u32) -> Result<usize, SysError> {
    let mut table = TIMERFD_TABLE.lock();
    for i in 0..table.entries.len() {
        if table.entries[i].is_none() {
            table.entries[i] = Some(TimerFdState {
                clockid,
                flags,
                it_value_ticks: 0,
                it_interval_ticks: 0,
                expirations: 0,
                waiting: false,
                armed: false,
            });
            return Ok(i);
        }
    }
    err!(SysError::FileTableFull)
}

pub fn free_timerfd_id(id: usize) {
    let mut table = TIMERFD_TABLE.lock();
    if id < table.entries.len() {
        table.entries[id] = None;
    }
}

pub fn timerfd_settime(id: usize, flags: u32, new_val: &Itimerspec, old_val: &mut Itimerspec) -> Result<(), SysError> {
    let mut table = TIMERFD_TABLE.lock();
    let Some(ref mut state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    *old_val = Itimerspec {
        it_interval: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    };

    if state.armed {
        let ticks_remaining = if state.it_value_ticks > crate::trap::current_ticks() {
            state.it_value_ticks - crate::trap::current_ticks()
        } else {
            0
        };
        let remaining_ms = ticks_remaining * 10;
        old_val.it_value.tv_sec = (remaining_ms / 1000) as i64;
        old_val.it_value.tv_nsec = ((remaining_ms % 1000) * 1_000_000) as i64;
    }

    let abs = (flags & TFD_TIMER_ABSTIME) != 0;
    let it_value_ticks = if abs {
        timespec_to_ticks(&new_val.it_value)
    } else {
        crate::trap::current_ticks() + timespec_to_ticks(&new_val.it_value)
    };

    state.it_value_ticks = it_value_ticks;
    state.it_interval_ticks = timespec_to_ticks(&new_val.it_interval);
    state.expirations = 0;
    state.armed = it_value_ticks > 0;

    Ok(())
}

pub fn timerfd_gettime(id: usize, curr: &mut Itimerspec) -> Result<(), SysError> {
    let table = TIMERFD_TABLE.lock();
    let Some(ref state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    if state.armed {
        let ticks_remaining = if state.it_value_ticks > crate::trap::current_ticks() {
            state.it_value_ticks - crate::trap::current_ticks()
        } else {
            0
        };
        let remaining_ms = ticks_remaining * 10;
        curr.it_value.tv_sec = (remaining_ms / 1000) as i64;
        curr.it_value.tv_nsec = ((remaining_ms % 1000) * 1_000_000) as i64;
    }

    let interval_ms = state.it_interval_ticks * 10;
    curr.it_interval.tv_sec = (interval_ms / 1000) as i64;
    curr.it_interval.tv_nsec = ((interval_ms % 1000) * 1_000_000) as i64;

    Ok(())
}

pub fn timerfd_read(id: usize, nonblock: bool) -> Result<u64, SysError> {
    let mut table = TIMERFD_TABLE.lock();
    let Some(ref mut state) = table.entries[id] else {
        err!(SysError::BadDescriptor)
    };

    if state.expirations > 0 {
        let val = state.expirations;
        state.expirations = 0;
        state.waiting = false;
        return Ok(val);
    }

    if nonblock || !state.armed {
        err!(SysError::ResourceUnavailable)
    }

    state.waiting = true;
    drop(state);
    let mut guard = table;
    loop {
        guard = crate::proc::sleep(crate::proc::Channel::TimerTick, guard);
        let val = {
            let state = guard.entries[id].as_mut().unwrap();
            if state.expirations > 0 {
                let v = state.expirations;
                state.expirations = 0;
                state.waiting = false;
                v
            } else if crate::proc::current_proc().is_killed() {
                state.waiting = false;
                return Err(SysError::Interrupted);
            } else {
                continue;
            }
        };
        return Ok(val);
    }
}

pub fn timerfd_readiness(id: usize) -> (bool, bool) {
    let table = TIMERFD_TABLE.lock();
    match &table.entries[id] {
        Some(state) => {
            let readable = state.expirations > 0;
            (readable, false)
        }
        None => (false, false),
    }
}

pub fn tick() {
    let now = crate::trap::current_ticks();
    let mut woken = false;

    {
        let mut table = TIMERFD_TABLE.lock();
        for entry in table.entries.iter_mut() {
            let Some(state) = entry else { continue };
            if !state.armed {
                continue;
            }
            if now < state.it_value_ticks {
                continue;
            }

            state.expirations += 1;

            if state.it_interval_ticks > 0 {
                state.it_value_ticks = now + state.it_interval_ticks;
            } else {
                state.armed = false;
            }

            if state.waiting {
                woken = true;
            }
        }
    }

    if woken {
        crate::proc::wakeup(crate::proc::Channel::TimerTick);
    }
}
