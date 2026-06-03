use super::time::Duration;

pub fn yield_now() {
    let _ = xv8_libc::sleep(0);
}

pub fn sleep(dur: Duration) {
    if dur.secs == 0 && dur.nanos == 0 {
        return;
    }

    let mut ticks = dur.secs.saturating_mul(100);
    if dur.nanos != 0 {
        ticks = ticks.saturating_add((dur.nanos as u64 + 9_999_999) / 10_000_000);
    }
    let _ = xv8_libc::sleep(ticks as usize);
}

pub fn available_parallelism() -> core::num::NonZeroUsize {
    core::num::NonZeroUsize::new(1).unwrap()
}
