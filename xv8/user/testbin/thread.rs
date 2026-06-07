#![no_std]
#![no_main]

use user::*;

const CLONE_VM: usize = 0x100;
const CLONE_SETTLS: usize = 0x800;
const CLONE_SIGHAND: usize = 0x80000;
const STACK_SIZE: usize = 0x4000;

fn alloc_stack() -> usize {
    let base = sbrk(STACK_SIZE as isize).expect("sbrk stack") as usize;
    // Leave room at the top for the function's stack frame (positive offsets from sp).
    // The compiler stores local variables at sp+N, where N can be as large as the
    // stack frame size of the calling function. With sp == data.size, sp+N would
    // exceed the page boundary and cause vmfault to reject the access.
    base + STACK_SIZE - 256
}

fn test_spawn_join() {
    let (r, w) = pipe().expect("pipe for join");
    let stack_top = alloc_stack();

    let flags = CLONE_VM | CLONE_SIGHAND;

    match clone(flags, stack_top).expect("clone") {
        0 => {
            let _ = close(r);
            let val: usize = 42;
            write(w, &val.to_le_bytes()).expect("child write");
            exit(0);
        }
        _pid => {
            let _ = close(w);
            let mut buf = [0u8; 8];
            let n = read(r, &mut buf).expect("parent read");
            assert_eq!(n, 8, "should read 8 bytes from child");
            let val = usize::from_le_bytes(buf);
            assert_eq!(val, 42, "child should return 42");
        }
    }
}

fn test_shared_memory() {
    let shared = sbrk(8).expect("sbrk") as *mut usize;
    unsafe { *shared = 0; }

    let (r, w) = pipe().expect("pipe");
    let stack_top = alloc_stack();

    let flags = CLONE_VM | CLONE_SIGHAND;

    match clone(flags, stack_top).expect("clone") {
        0 => {
            let _ = close(r);
            unsafe { *shared += 1; }
            write(w, &[1u8]).expect("child signal");
            exit(0);
        }
        _pid => {
            let _ = close(w);
            let mut buf = [0u8; 1];
            let n = read(r, &mut buf).expect("parent read");
            assert_eq!(n, 1, "should receive child signal");
            assert_eq!(unsafe { *shared }, 1, "shared counter should be 1");
        }
    }
}

fn test_multi_spawn() {
    const NTHREADS: usize = 4;

    let mut read_fds = [Fd::from_raw(0); NTHREADS];

    for i in 0..NTHREADS {
        let (r, w) = pipe().expect("pipe");
        read_fds[i] = r;
        let stack_top = alloc_stack();

        let flags = CLONE_VM | CLONE_SIGHAND;

        match clone(flags, stack_top).expect("clone") {
            0 => {
                let _ = close(r);
                let val = (i + 1) * 10;
                write(w, &val.to_le_bytes()).expect("child write");
                exit(0);
            }
            _pid => {
                let _ = close(w);
            }
        }
    }

    for i in 0..NTHREADS {
        let mut buf = [0u8; 8];
        let n = read(read_fds[i], &mut buf).expect("parent read");
        assert_eq!(n, 8, "should read from thread {i}");
        let val = usize::from_le_bytes(buf);
        assert_eq!(val, (i + 1) * 10, "thread {i} value mismatch");
    }
}

fn test_clone_set_tls() {
    let expected_tls = 0x12345678usize;

    let (r, w) = pipe().expect("pipe");
    let stack_top = alloc_stack();

    let flags = CLONE_VM | CLONE_SETTLS;

    match clone_with_tls(flags, stack_top, 0, expected_tls).expect("clone") {
        0 => {
            let _ = close(r);
            let tp: usize;
            unsafe { core::arch::asm!("mv {}, tp", out(reg) tp) };
            let val = tp;
            write(w, &val.to_le_bytes()).expect("child write");
            exit(0);
        }
        _pid => {
            let _ = close(w);
            let mut buf = [0u8; 8];
            let n = read(r, &mut buf).expect("parent read");
            assert_eq!(n, 8, "should read 8 bytes from child");
            let tp = usize::from_le_bytes(buf);
            assert_eq!(tp, expected_tls, "tp should match expected TLS value");
        }
    }
}

fn test_park_unpark() {
    let (park_r, park_w) = pipe().expect("park pipe");
    let (done_r, done_w) = pipe().expect("done pipe");
    let stack_top = alloc_stack();
    let flags = CLONE_VM | CLONE_SIGHAND;

    match clone(flags, stack_top).expect("clone") {
        0 => {
            let _ = close(park_w);
            let _ = close(done_r);
            let mut _buf = [0u8; 1];
            let n = read(park_r, &mut _buf).expect("child parked");
            assert_eq!(n, 1, "park read should receive 1 byte");
            write(done_w, &[1u8]).expect("child done signal");
            exit(0);
        }
        _pid => {
            let _ = close(park_r);
            let _ = close(done_w);
            write(park_w, &[1u8]).expect("parent unpark");
            let mut buf = [0u8; 1];
            let n = read(done_r, &mut buf).expect("parent wait done");
            assert_eq!(n, 1, "should read child done signal");
        }
    }
}

fn test_thread_sleep() {
    let ticks = 5;
    sleep(ticks).expect("sleep failed");
}

fn test_multi_park() {
    const N: usize = 4;
    let mut park_w_fds = [Fd::from_raw(0); N];
    let mut done_r_fds = [Fd::from_raw(0); N];

    for i in 0..N {
        let (park_r, park_w) = pipe().expect("park pipe");
        let (done_r, done_w) = pipe().expect("done pipe");
        park_w_fds[i] = park_w;
        done_r_fds[i] = done_r;
        let stack_top = alloc_stack();
        let flags = CLONE_VM | CLONE_SIGHAND;

        match clone(flags, stack_top).expect("clone") {
            0 => {
                let _ = close(park_w);
                let _ = close(done_r);
                let mut _buf = [0u8; 1];
                let n = read(park_r, &mut _buf).expect("child parked");
                assert_eq!(n, 1, "child {i} unparked");
                write(done_w, &[(i + 1) as u8]).expect("child done");
                exit(0);
            }
            _ => {
                let _ = close(park_r);
                let _ = close(done_w);
            }
        }
    }

    for i in 0..N {
        write(park_w_fds[i], &[1u8]).expect("unpark");
    }

    for i in 0..N {
        let mut buf = [0u8; 1];
        let n = read(done_r_fds[i], &mut buf).expect("parent read done");
        assert_eq!(n, 1, "should read from child {i}");
        assert_eq!(buf[0], (i + 1) as u8, "child {i} value mismatch");
    }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_spawn_join();
    test_shared_memory();
    test_multi_spawn();
    test_clone_set_tls();
    test_park_unpark();
    test_thread_sleep();
    test_multi_park();
}
