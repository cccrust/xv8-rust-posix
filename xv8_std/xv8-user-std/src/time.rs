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
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct SystemTime { secs: u64, nanos: u32 }

pub const UNIX_EPOCH: SystemTime = SystemTime { secs: 0, nanos: 0 };

impl SystemTime {
    pub fn now() -> Self { SystemTime { secs: 0, nanos: 0 } }
    pub fn duration_since(&self, _earlier: SystemTime) -> Result<Duration, ()> {
        Ok(Duration::new(0, 0))
    }
    pub fn elapsed(&self) -> Result<Duration, ()> { Ok(Duration::new(0, 0)) }
}
