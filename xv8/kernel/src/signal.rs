use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const SIGDEF: usize = 0;
pub const SIGIGN: usize = 1;
pub const SIGACT: usize = 2;

pub const SIGNAL_MAX: usize = 32;

pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

pub const SIGALRM: usize = 14;
pub const SIGVTALRM: usize = 26;
pub const SIGPROF: usize = 27;
pub const SIGIO: usize = 29;
pub const SIGPIPE: usize = 13;
pub const SIGTERM: usize = 15;
pub const SIGINT: usize = 2;
pub const SIGKILL: usize = 9;
pub const SIGSTOP: usize = 19;
pub const SIGCONT: usize = 18;
pub const SIGHUP: usize = 1;
pub const SIGQUIT: usize = 3;
pub const SIGABRT: usize = 6;
pub const SIGSEGV: usize = 11;
pub const SIGUSR1: usize = 10;
pub const SIGUSR2: usize = 12;

pub const SA_NOCLDSTOP: u32 = 0x00000001;
pub const SA_NOCLDWAIT: u32 = 0x00000002;
pub const SA_SIGINFO: u32 = 0x00000004;
pub const SA_RESTART: u32 = 0x10000000;
pub const SA_NODEFER: u32 = 0x20000000;
pub const SA_RESETHAND: u32 = 0x40000000;

pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: u32 = 2;
pub const CLOCK_THREAD_CPUTIME_ID: u32 = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigFrame {
    pub signo: i32,
    pub pad: i32,
    pub epc: u64,
    pub ra: u64,
    pub sp: u64,
    pub gp: u64,
    pub tp: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub s0: u64,
    pub s1: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub a4: u64,
    pub a5: u64,
    pub a6: u64,
    pub a7: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,
    pub oldmask: u64,
}

/// Returns the default action for a signal (true = kill, false = ignore).
pub fn sig_default_action(sig: usize) -> bool {
    matches!(sig, SIGKILL | SIGSEGV | SIGABRT | SIGQUIT | SIGTERM | SIGINT | SIGHUP | SIGPIPE | SIGALRM | SIGUSR1 | SIGUSR2 | SIGVTALRM | SIGPROF | SIGIO)
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SigAction {
    pub handler: usize,
    pub flags: u32,
    pub mask: u32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SigInfo {
    pub si_signo: u32,
    pub si_code: u32,
    pub si_errno: u32,
    pub si_addr: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    None,
    Pending(u32),
    Delivered,
}

#[derive(Debug)]
pub struct SignalState {
    pub pending: AtomicUsize,
    pub blocked: AtomicUsize,
    pub in_handler: AtomicBool,
    pub alarm_time: AtomicUsize,
}

impl Default for SignalState {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalState {
    pub const fn new() -> Self {
        Self {
            pending: AtomicUsize::new(0),
            blocked: AtomicUsize::new(0),
            in_handler: AtomicBool::new(false),
            alarm_time: AtomicUsize::new(0),
        }
    }

    pub fn get_pending(&self) -> usize {
        self.pending.load(Ordering::Relaxed)
    }

    pub fn get_blocked(&self) -> usize {
        self.blocked.load(Ordering::Relaxed)
    }

    pub fn has_pending(&self) -> bool {
        self.pending.load(Ordering::Relaxed) != 0
    }

    pub fn get_pending_signal(&self) -> Option<usize> {
        let sig = self.pending.load(Ordering::Relaxed) & !self.blocked.load(Ordering::Relaxed);
        if sig == 0 {
            None
        } else {
            Some(sig.trailing_zeros() as usize + 1)
        }
    }

    pub fn clear_signal(&self, sig: usize) {
        if sig > 0 && sig <= SIGNAL_MAX {
            self.pending.fetch_and(!(1 << (sig - 1)), Ordering::Relaxed);
        }
    }

    pub fn send_signal(&self, sig: usize) {
        if sig > 0 && sig <= SIGNAL_MAX {
            self.pending.fetch_or(1 << (sig - 1), Ordering::Relaxed);
        }
    }

    pub fn get_alarm_time(&self) -> usize {
        self.alarm_time.load(Ordering::Relaxed)
    }

    pub fn set_alarm_time(&self, time: usize) {
        self.alarm_time.store(time, Ordering::Relaxed);
    }
}

pub static ALARM_PENDING: AtomicBool = AtomicBool::new(false);

pub fn get_time_us() -> usize {
    let ticks = *crate::trap::TICKS.lock();
    ticks * 10000
}

pub fn get_time_ms() -> usize {
    let ticks = *crate::trap::TICKS.lock();
    ticks * 10
}