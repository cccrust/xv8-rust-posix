#![no_std]
#![no_main]

use user::*;

fn test_pidfd_open_self() {
    let self_pid = getpid();
    let fd = raw::pidfd_open(self_pid, 0);
    assert!(fd >= 0, "pidfd_open(self={self_pid}) failed: {fd}");
    let fd = Fd::from_raw(fd as usize);
    close(fd).expect("close pidfd");
    println!("ok pidfd_open_self");
}

fn test_pidfd_open_child() {
    match fork().expect("fork") {
        0 => {
            loop {}
        }
        child_pid => {
            let fd = raw::pidfd_open(child_pid, 0);
            assert!(fd >= 0, "pidfd_open(child={child_pid}) failed: {fd}");
            let fd = Fd::from_raw(fd as usize);

            kill(child_pid as usize).expect("kill child");
            let mut status = 0;
            wait(&mut status).expect("wait for child");

            close(fd).expect("close pidfd");
            println!("ok pidfd_open_child");
        }
    }
}

fn test_pidfd_invalid_pid() {
    let fd = raw::pidfd_open(99999, 0);
    assert!(fd < 0, "pidfd_open(invalid) should fail, got {fd}");
    println!("ok pidfd_invalid_pid");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_pidfd_open_self();
    test_pidfd_open_child();
    test_pidfd_invalid_pid();
}
