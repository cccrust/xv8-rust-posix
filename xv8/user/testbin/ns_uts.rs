#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let my_hostname = b"parent-host";
    sethostname(my_hostname).unwrap();

    // clone with CLONE_NEWUTS (separate UTS namespace)
    // The child's UTS namespace starts as a COPY of the parent's
    let flags = CLONE_NEWUTS | 0x11;
    match clone(flags, 0) {
        Ok(0) => {
            // child: new UTS ns initially has SAME hostname (copy-on-unshare)
            let mut buf = [0u8; 64];
            let n = gethostname(&mut buf).unwrap();
            let child_hostname = &buf[..n];
            // verify child inherited parent's hostname
            if child_hostname != my_hostname {
                println!(
                    "FAIL: child initial hostname mismatch: got '{:?}'",
                    core::str::from_utf8(child_hostname)
                );
                exit(1);
            }
            // child sets its own hostname — should NOT affect parent
            let child_name = b"child-host";
            sethostname(child_name).unwrap();
            // verify child sees its own hostname
            let mut buf2 = [0u8; 64];
            let n2 = gethostname(&mut buf2).unwrap();
            if &buf2[..n2] != child_name {
                println!("FAIL: child hostname didn't stick");
                exit(1);
            }
            exit(0);
        }
        Ok(child_pid) => {
            // parent: verify hostname is STILL the original (child's changes isolated)
            let mut buf = [0u8; 64];
            let n = gethostname(&mut buf).unwrap();
            if &buf[..n] != my_hostname {
                println!(
                    "FAIL: parent hostname changed after child CLONE_NEWUTS: got '{:?}'",
                    core::str::from_utf8(&buf[..n])
                );
                exit(1);
            }
            let mut code = 0;
            match wait(&mut code) {
                Ok(pid) if pid == child_pid => {}
                other => {
                    println!("wait: {:?} (expected pid={})", other, child_pid);
                    exit(1);
                }
            }
            if code != 0 {
                println!("child failed (code {})", code);
                exit(1);
            }
            println!("ns_uts test passed");
        }
        Err(e) => {
            println!("clone failed: {:?}", e);
            exit(1);
        }
    }
}
