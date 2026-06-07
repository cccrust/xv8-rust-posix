#![no_std]
#![no_main]

use user::*;

const CLONE_VM: usize = 0x100;
const CLONE_SETTLS: usize = 0x800;
const CLONE_SIGHAND: usize = 0x80000;
const STACK_SIZE: usize = 0x4000;

#[repr(C)]
struct Tcb {
    _park: u32,
    name: usize,
    _args: usize,
}

fn alloc_stack(size: usize) -> usize {
    let base = sbrk(size as isize).expect("sbrk") as usize;
    base + size - 256
}

fn test_builder_pattern() {
    let (r, w) = pipe().expect("pipe");
    let stack_top = alloc_stack(STACK_SIZE);
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
            assert_eq!(n, 8, "should read 8 bytes");
            let val = usize::from_le_bytes(buf);
            assert_eq!(val, 42, "child should return 42");
        }
    }
}

fn test_named_thread() {
    let (r, w) = pipe().expect("pipe");
    let tcb = sbrk(64).expect("sbrk tcb") as *mut Tcb;
    let name_bytes = b"test-thread\0";
    let name_mem = sbrk(name_bytes.len() as isize).expect("sbrk name") as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name_mem, name_bytes.len());
        (*tcb).name = name_mem as usize;
    }
    let stack_top = alloc_stack(STACK_SIZE);
    let flags = CLONE_VM | CLONE_SETTLS;

    match clone_with_tls(flags, stack_top, 0, tcb as usize).expect("clone") {
        0 => {
            let _ = close(r);
            let tp: usize;
            unsafe { core::arch::asm!("mv {}, tp", out(reg) tp) };
            let child_tcb = tp as *const Tcb;
            let name_ptr = unsafe { (*child_tcb).name };
            let mut name_buf = [0u8; 64];
            if name_ptr != 0 {
                let mut i = 0;
                while i < name_buf.len() {
                    let b = unsafe { *(name_ptr as *const u8).add(i) };
                    name_buf[i] = b;
                    if b == 0 { break; }
                    i += 1;
                }
            }
            write(w, &name_buf).expect("child write name");
            exit(0);
        }
        _pid => {
            let _ = close(w);
            let mut buf = [0u8; 64];
            let n = read(r, &mut buf).expect("parent read");
            let name_len = buf.iter().position(|&b| b == 0).unwrap_or(n);
            assert_eq!(&buf[..name_len], b"test-thread", "thread name should match");
        }
    }
}

fn test_stack_size() {
    let (r, w) = pipe().expect("pipe");
    let stack_top = alloc_stack(8192);
    let flags = CLONE_VM | CLONE_SIGHAND;

    match clone(flags, stack_top).expect("clone") {
        0 => {
            let _ = close(r);
            write(w, &[1u8]).expect("child signal");
            exit(0);
        }
        _pid => {
            let _ = close(w);
            let mut buf = [0u8; 1];
            let n = read(r, &mut buf).expect("parent read");
            assert_eq!(n, 1, "should signal from child with 8K stack");
        }
    }
}

fn test_multi_named() {
    const N: usize = 3;
    let mut read_fds = [Fd::from_raw(0); N];

    for i in 0..N {
        let (r, w) = pipe().expect("pipe");
        read_fds[i] = r;

        let tcb = sbrk(64).expect("sbrk tcb") as *mut Tcb;
        let name = [b"thread-1\0", b"thread-2\0", b"thread-3\0"][i];
        let name_mem = sbrk(16).expect("sbrk name") as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(name.as_ptr(), name_mem, name.len());
            (*tcb).name = name_mem as usize;
        }
        let stack_top = alloc_stack(STACK_SIZE);
        let flags = CLONE_VM | CLONE_SETTLS;

        match clone_with_tls(flags, stack_top, 0, tcb as usize).expect("clone") {
            0 => {
                let _ = close(r);
                let tp: usize;
                unsafe { core::arch::asm!("mv {}, tp", out(reg) tp) };
                let child_tcb = tp as *const Tcb;
                let name_ptr = unsafe { (*child_tcb).name };
                let mut name_buf = [0u8; 32];
                if name_ptr != 0 {
                    let mut j = 0;
                    while j < name_buf.len() {
                        let b = unsafe { *(name_ptr as *const u8).add(j) };
                        name_buf[j] = b;
                        if b == 0 { break; }
                        j += 1;
                    }
                }
                write(w, &name_buf).expect("child write");
                exit(0);
            }
            _pid => {
                let _ = close(w);
            }
        }
    }

    let expected = [b"thread-1\0", b"thread-2\0", b"thread-3\0"];
    for i in 0..N {
        let mut buf = [0u8; 32];
        let n = read(read_fds[i], &mut buf).expect("parent read");
        let name_len = buf.iter().position(|&b| b == 0).unwrap_or(n);
        assert_eq!(&buf[..name_len], &expected[i][..name_len], "thread {i} name");
    }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_builder_pattern();
    test_named_thread();
    test_stack_size();
    test_multi_named();
}
