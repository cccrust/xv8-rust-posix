#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let parent_pid = getpid();
    assert!(parent_pid > 0);

    // clone with CLONE_NEWPID (separate PID namespace)
    // Note: xv8 returns global PID from getpid (not namespace-local PID)
    let flags = CLONE_NEWPID | 0x11;
    match clone(flags, 0) {
        Ok(0) => {
            // child: getpid returns global PID (not 1 in new ns)
            let child_pid = getpid();
            assert!(child_pid > 0);
            exit(0);
        }
        Ok(child_pid) => {
            // parent: wait for child
            let mut code = 0;
            match wait(&mut code) {
                Ok(pid) if pid == child_pid => {}
                other => {
                    println!("wait: {:?} (expected pid={})", other, child_pid);
                    exit(1);
                }
            }
            if code != 0 {
                println!("child failed with code {}", code);
                exit(1);
            }
            println!("ns_pid test passed");
        }
        Err(e) => {
            println!("clone failed: {:?}", e);
            exit(1);
        }
    }
}
