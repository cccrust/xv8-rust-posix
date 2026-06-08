#![no_std]
#![no_main]

use user::*;

const ALL_TESTS: &[&str] = &[
    "/_fs", "/_pipe", "/_proc", "/_fd", "/_sbrk", "/_cow",
    "/_net", "/_syscall", "/_neteth", "/_netdns", "/_tcpecho", "/_nettools", "/_http",
    "/_async", "/_httpepoll", "/_axum",
    "/_shtest",
    "/_thread",
    "/_thread_v3",
    "/_eventfd",
    "/_memfd_create",
    "/_pidfd",
    "/_splice",
    "/_getrandom",
    "/_close_range",
    "/_inotify",
    "/_signalfd",
    "/_timerfd",
    "/_ns_pid",
    "/_ns_uts",
    "/_setns",
    "/_cgroup",
    "/_capability",
    "/_seccomp",
    "/_overlay",
    "/_veth",
    "/_pivot_root",
    "/_container",
];

static mut FILTER_BUF: [u8; 256] = [0; 256];
static mut FILTER_LEN: usize = 0;

fn load_filter() {
    let fd = match open("test_args", OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => return,
    };
    unsafe {
        let ptr: *mut u8 = &raw mut FILTER_BUF as *mut u8;
        FILTER_LEN = match raw::read(fd.as_raw(), ptr, 256) {
            n if n > 0 => n as usize,
            _ => 0,
        };
    }
    let _ = close(fd);
}

fn has_filter() -> bool {
    unsafe { (&raw const FILTER_LEN as *const usize).read() > 0 }
}

fn test_matches(name: &str) -> bool {
    unsafe {
        let buf: *const u8 = &raw const FILTER_BUF as *const u8;
        let len = FILTER_LEN;
        if len == 0 {
            return true;
        }
        let test_short = &name[2..];
        let mut start = 0;
        for i in 0..=len {
            let c = if i == len { b',' } else { *buf.add(i) };
            if c == b',' || c == b'\n' {
                if start < i {
                    let slice = core::slice::from_raw_parts(buf.add(start), i - start);
                    let frag = core::str::from_utf8(slice).unwrap_or("").trim();
                    if !frag.is_empty() && test_short == frag {
                        return true;
                    }
                }
                start = i + 1;
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    load_filter();
    let tests: &[&str] = ALL_TESTS;

    let count = tests.iter().filter(|t| !has_filter() || test_matches(*t)).count();
    println!("running {} tests\n", count);

    let mut passed = 0;
    let mut failed = 0;

    for name in tests {
        if has_filter() && !test_matches(name) { continue; }

        print!("test {} ... ", &name[2..]);

        if fork().expect("fork") == 0 {
            exec(name, &[&name[2..]]);
            unreachable!("exec failed");
        }

        let mut code = 0;
        wait(&mut code).expect("wait failed");

        if code == 0 {
            println!("ok");
            passed += 1;
        } else {
            println!("FAILED");
            failed += 1;
        }
    }

    println!(
        "\ntest result: {}. {} passed; {} failed",
        if failed == 0 { "ok" } else { "FAILED" },
        passed,
        failed,
    );

    poweroff(if failed == 0 { 0 } else { 1 });
}
