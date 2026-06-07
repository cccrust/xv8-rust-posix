#![no_std]
#![no_main]

use user::*;

fn test_eventfd_create_read_write() {
    let fd = raw::eventfd2(0, 0);
    assert!(fd >= 0, "eventfd2(0, 0) failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    let val: u64 = 42;
    let n = write(fd, unsafe {
        core::slice::from_raw_parts(&val as *const _ as *const u8, 8)
    }).expect("write 42 to eventfd");
    assert_eq!(n, 8, "write returned {n}");

    let mut buf = [0u8; 8];
    let n = read(fd, &mut buf).expect("read from eventfd");
    assert_eq!(n, 8, "read returned {n}");
    let result = u64::from_le_bytes(buf);
    assert_eq!(result, 42, "read back {result} instead of 42");

    close(fd).expect("close eventfd");
    println!("ok eventfd_create_read_write");
}

fn test_eventfd_semaphore() {
    let fd = raw::eventfd2(3, kernel::abi::EFD_SEMAPHORE);
    assert!(fd >= 0, "eventfd2(3, EFD_SEMAPHORE) failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    let mut buf = [0u8; 8];
    let n = read(fd, &mut buf).expect("semaphore read");
    assert_eq!(n, 8, "semaphore read returned {n}");
    let result = u64::from_le_bytes(buf);
    assert_eq!(result, 1, "semaphore read {result} instead of 1");

    let n = read(fd, &mut buf).expect("semaphore read 2");
    assert_eq!(n, 8, "second read returned {n}");
    let result = u64::from_le_bytes(buf);
    assert_eq!(result, 1, "second semaphore read {result} instead of 1");

    close(fd).expect("close semaphore eventfd");
    println!("ok eventfd_semaphore");
}

fn test_eventfd_epoll() {
    let epfd = epoll_create1(0).expect("epoll_create1");
    let fd = raw::eventfd2(0, 0);
    assert!(fd >= 0, "eventfd2 failed");
    let fd = Fd::from_raw(fd as usize);

    let event = kernel::abi::EpollEvent {
        events: kernel::abi::EPOLLIN as u32,
        data: 0xdead,
    };
    epoll_ctl(epfd, kernel::abi::EPOLL_CTL_ADD, fd, Some(&event)).expect("epoll_ctl add");

    let mut events = [kernel::abi::EpollEvent { events: 0, data: 0 }; 1];
    let n = epoll_wait(epfd, &mut events, 0).expect("epoll_wait with timeout 0 before write");
    assert_eq!(n, 0, "epoll should not fire before write");

    let val: u64 = 1;
    write(fd, unsafe {
        core::slice::from_raw_parts(&val as *const _ as *const u8, 8)
    }).expect("write to trigger epoll");

    let n = epoll_wait(epfd, &mut events, 10).expect("epoll_wait after write");
    assert_eq!(n, 1, "epoll should fire after write");
    let revents: u32 = unsafe { core::ptr::addr_of!(events[0].events).read_unaligned() };
    assert!(revents & kernel::abi::EPOLLIN as u32 != 0,
            "expected EPOLLIN, got {:x}", revents);

    close(fd).expect("close eventfd");
    close(epfd).expect("close epoll");
    println!("ok eventfd_epoll");
}

fn test_eventfd_nonblock() {
    let fd = raw::eventfd2(0, kernel::abi::EFD_NONBLOCK);
    assert!(fd >= 0, "eventfd2(NONBLOCK) failed: {}", fd);
    let fd = Fd::from_raw(fd as usize);

    let mut buf = [0u8; 8];
    let ret = read(fd, &mut buf);
    assert!(ret.is_err(), "read on empty nonblock eventfd should fail");

    close(fd).expect("close");
    println!("ok eventfd_nonblock");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_eventfd_create_read_write();
    test_eventfd_semaphore();
    test_eventfd_epoll();
    test_eventfd_nonblock();
}
