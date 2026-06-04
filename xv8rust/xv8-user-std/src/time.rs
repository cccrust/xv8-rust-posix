#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

impl Duration {
    pub const fn new(secs: u64, nanos: u32) -> Self {
        Duration { secs, nanos }
    }
    pub fn as_secs(&self) -> u64 { self.secs }
    pub fn as_millis(&self) -> u64 { self.secs * 1000 + (self.nanos / 1_000_000) as u64 }
    pub fn as_micros(&self) -> u64 { self.secs * 1_000_000 + (self.nanos / 1_000) as u64 }
    pub fn as_nanos(&self) -> u128 { (self.secs as u128) * 1_000_000_000 + (self.nanos as u128) }
    pub const fn from_secs(secs: u64) -> Self { Duration { secs, nanos: 0 } }
    pub const fn from_millis(millis: u64) -> Self {
        Duration { secs: millis / 1000, nanos: ((millis % 1000) * 1_000_000) as u32 }
    }
    pub fn subsec_nanos(&self) -> u32 { self.nanos }
    pub fn subsec_millis(&self) -> u32 { self.nanos / 1_000_000 }
}

impl Default for Duration {
    fn default() -> Self { Duration::new(0, 0) }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct SystemTime {
    secs: u64,
    nanos: u32,
}

pub const UNIX_EPOCH: SystemTime = SystemTime { secs: 0, nanos: 0 };

impl SystemTime {
    pub fn now() -> Self {
        let secs = xv8_libc::time(core::ptr::null_mut());
        SystemTime { secs: secs.max(0) as u64, nanos: 0 }
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> {
        if *self < earlier {
            return Err(());
        }

        let lhs = self.secs.saturating_mul(1_000_000_000) + self.nanos as u64;
        let rhs = earlier.secs.saturating_mul(1_000_000_000) + earlier.nanos as u64;
        let diff = lhs.saturating_sub(rhs);
        Ok(Duration::new(diff / 1_000_000_000, (diff % 1_000_000_000) as u32))
    }

    pub fn elapsed(&self) -> Result<Duration, ()> {
        SystemTime::now().duration_since(*self)
    }
}

impl core::ops::Add<Duration> for SystemTime {
    type Output = SystemTime;
    fn add(self, other: Duration) -> SystemTime {
        let nanos = self.nanos as u64 + other.nanos as u64;
        SystemTime {
            secs: self.secs + other.secs + nanos / 1_000_000_000,
            nanos: (nanos % 1_000_000_000) as u32,
        }
    }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Instant {
    secs: u64,
    nanos: u32,
}

fn ticks_to_duration(ticks: usize) -> Duration {
    let secs = (ticks / 100) as u64;
    let nanos = ((ticks % 100) as u32) * 10_000_000;
    Duration::new(secs, nanos)
}

fn duration_to_ticks(dur: Duration) -> usize {
    let mut ticks = dur.secs.saturating_mul(100);
    ticks = ticks.saturating_add((dur.nanos as u64 + 9_999_999) / 10_000_000);
    ticks as usize
}

impl Instant {
    pub fn now() -> Self {
        let ticks = xv8_libc::uptime();
        let ticks = if ticks < 0 { 0 } else { ticks as usize };
        let d = ticks_to_duration(ticks);
        Instant { secs: d.secs, nanos: d.nanos }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        if *self < earlier {
            return Duration::new(0, 0);
        }

        let lhs = self.secs.saturating_mul(1_000_000_000) + self.nanos as u64;
        let rhs = earlier.secs.saturating_mul(1_000_000_000) + earlier.nanos as u64;
        let diff = lhs.saturating_sub(rhs);
        Duration::new(diff / 1_000_000_000, (diff % 1_000_000_000) as u32)
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(*self)
    }

    pub fn checked_add(&self, other: Duration) -> Option<Instant> {
        Some(*self + other)
    }

    pub fn checked_sub(&self, other: Duration) -> Option<Instant> {
        Some(*self - other)
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, other: Duration) -> Instant {
        let current = duration_to_ticks(Duration::new(self.secs, self.nanos));
        let delta = duration_to_ticks(other);
        let ticks = current.saturating_sub(delta);
        let d = ticks_to_duration(ticks);
        Instant { secs: d.secs, nanos: d.nanos }
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, other: Duration) -> Instant {
        let current = duration_to_ticks(Duration::new(self.secs, self.nanos));
        let ticks = current.saturating_add(duration_to_ticks(other));
        let d = ticks_to_duration(ticks);
        Instant { secs: d.secs, nanos: d.nanos }
    }
}
