#![no_std]
#![no_main]

use user::*;

fn test_timerfd_create() {
    let fd = timerfd_create(CLOCK_MONOTONIC as i32, 0).expect("timerfd_create");
    assert!(fd.as_raw() > 0, "timerfd fd should be > 0");
    let _ = close(fd);
    println!("ok");
}

fn test_timerfd_settime_gettime() {
    let fd = timerfd_create(CLOCK_MONOTONIC as i32, 0).expect("timerfd_create");

    let mut curr = Itimerspec::default();
    timerfd_gettime(fd, &mut curr as *mut _ as usize).expect("timerfd_gettime (disarmed)");
    assert_eq!(curr.it_value.tv_sec, 0, "disarmed timer should have 0 it_value");
    assert_eq!(curr.it_value.tv_nsec, 0, "disarmed timer should have 0 it_value");

    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 50_000_000 },
    };
    timerfd_settime(fd, 0, &new_val as *const _ as usize).expect("timerfd_settime");

    timerfd_gettime(fd, &mut curr as *mut _ as usize).expect("timerfd_gettime (armed)");
    assert!(curr.it_value.tv_sec > 0 || curr.it_value.tv_nsec > 0, "armed timer should have nonzero it_value");

    let _ = close(fd);
    println!("ok");
}

fn test_timerfd_read() {
    let fd = timerfd_create(CLOCK_MONOTONIC as i32, 0).expect("timerfd_create");

    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 20_000_000 },
    };
    timerfd_settime(fd, 0, &new_val as *const _ as usize).expect("timerfd_settime");

    let mut buf = [0u8; 8];
    let n = read(fd, &mut buf).expect("read timerfd");
    assert_eq!(n, 8, "should read 8 bytes (u64)");

    let expirations = u64::from_ne_bytes(buf);
    assert!(expirations >= 1, "should have at least 1 expiration, got {}", expirations);

    let _ = close(fd);
    println!("ok");
}

fn test_timerfd_periodic() {
    let fd = timerfd_create(CLOCK_MONOTONIC as i32, 0).expect("timerfd_create");

    let new_val = Itimerspec {
        it_interval: Timespec { tv_sec: 0, tv_nsec: 10_000_000 },
        it_value: Timespec { tv_sec: 0, tv_nsec: 10_000_000 },
    };
    timerfd_settime(fd, 0, &new_val as *const _ as usize).expect("timerfd_settime");

    let mut buf = [0u8; 8];
    let n = read(fd, &mut buf).expect("first read");
    assert_eq!(n, 8);
    let e1 = u64::from_ne_bytes(buf);
    assert!(e1 >= 1, "first expiration count >= 1, got {}", e1);

    let n = read(fd, &mut buf).expect("second read");
    assert_eq!(n, 8);
    let e2 = u64::from_ne_bytes(buf);
    assert!(e2 >= 1, "second expiration count >= 1, got {}", e2);

    let n = read(fd, &mut buf).expect("third read");
    assert_eq!(n, 8);
    let e3 = u64::from_ne_bytes(buf);
    assert!(e3 >= 1, "third expiration count >= 1, got {}", e3);

    let _ = close(fd);
    println!("ok");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("test timerfd_create ... ");
    test_timerfd_create();

    println!("test timerfd_settime_gettime ... ");
    test_timerfd_settime_gettime();

    println!("test timerfd_read ... ");
    test_timerfd_read();

    println!("test timerfd_periodic ... ");
    test_timerfd_periodic();
}
