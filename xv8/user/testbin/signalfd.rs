#![no_std]
#![no_main]

use user::*;

fn test_signalfd_create() {
    let fd = signalfd4(0, 0).expect("signalfd4");
    assert!(fd.as_raw() > 0, "signalfd fd should be > 0");
    let _ = close(fd);
    println!("ok");
}

fn test_signalfd_read_signal() {
    let sigusr1_mask = 1u32 << (kernel::abi::SIGUSR1 - 1);
    let fd = signalfd4(sigusr1_mask, 0).expect("signalfd4");

    let pgrp = getpgid(0).expect("getpgid");
    killpg(pgrp, kernel::abi::SIGUSR1 as usize).expect("killpg");

    let mut buf = [0u8; 128];
    let n = read(fd, &mut buf).expect("read signalfd");
    assert_eq!(n, 128, "should read 128 bytes");

    let info: &kernel::abi::SignalfdSiginfo = unsafe { &*(buf.as_ptr() as *const kernel::abi::SignalfdSiginfo) };
    assert_eq!(info.ssi_signo, kernel::abi::SIGUSR1 as u32, "wrong signal number");

    let _ = close(fd);
    println!("ok");
}

fn test_signalfd_poll() {
    let sigusr2_mask = 1u32 << (kernel::abi::SIGUSR2 - 1);
    let fd = signalfd4(sigusr2_mask, 0).expect("signalfd4");

    let mut pfd = kernel::abi::PollFd {
        fd: fd.as_raw() as i32,
        events: kernel::abi::POLLIN as i16,
        revents: 0,
    };
    let ret = poll(core::slice::from_mut(&mut pfd), 0).expect("poll");
    assert_eq!(ret, 0, "no events without signal");

    let pgrp = getpgid(0).expect("getpgid");
    killpg(pgrp, kernel::abi::SIGUSR2 as usize).expect("killpg");

    let ret = poll(core::slice::from_mut(&mut pfd), 0).expect("poll");
    assert_eq!(ret, 1, "should have 1 event");
    assert!(pfd.revents & kernel::abi::POLLIN as i16 != 0, "should be readable");

    let _ = close(fd);
    println!("ok");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("test signalfd_create ... ");
    test_signalfd_create();

    println!("test signalfd_read_signal ... ");
    test_signalfd_read_signal();

    println!("test signalfd_poll ... ");
    test_signalfd_poll();
}
