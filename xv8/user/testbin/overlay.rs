#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    // Create fresh dirs and mount
    println!("overlay test: stage 1 - fresh dirs");
    let _ = mkdir("/mnt");
    check(mkdir("/mnt/merged"), "mkdir merged");
    check(mkdir("/mnt/upper"), "mkdir upper");
    check(mkdir("/mnt/lower"), "mkdir lower");

    // Check that /mnt, /mnt/merged, /mnt/upper, /mnt/lower all open
    let fd = check(open("/mnt", OpenFlag::READ_ONLY), "open /mnt");
    check(close(fd), "close /mnt");
    let fd = check(open("/mnt/merged", OpenFlag::READ_ONLY), "open /mnt/merged");
    check(close(fd), "close /mnt/merged");
    let fd = check(open("/mnt/upper", OpenFlag::READ_ONLY), "open /mnt/upper");
    check(close(fd), "close /mnt/upper");
    let fd = check(open("/mnt/lower", OpenFlag::READ_ONLY), "open /mnt/lower");
    check(close(fd), "close /mnt/lower");

    check(open("/mnt/lower/a", OpenFlag::CREATE), "create lower/a");
    check(open("/mnt/lower/b", OpenFlag::CREATE), "create lower/b");
    check(open("/mnt/lower/c", OpenFlag::CREATE), "create lower/c");

    println!("overlay: before syscall");
    match overlay_mount("/mnt/merged", "/mnt/upper", "/mnt/lower") {
        Ok(_) => println!("stage 1 ok"),
        Err(e) => {
            print!("stage1 failed code={} ", e.as_code());
            exit_with_msg("stage1");
        }
    }

    // Read through: /mnt/merged/a should come from lower
    let fd = check(open("/mnt/merged/a", OpenFlag::READ_ONLY), "open /mnt/merged/a");
    check(close(fd), "close");

    // Create new file in merged view (should go to upper)
    check(open("/mnt/merged/d", OpenFlag::CREATE), "create /mnt/merged/d");

    // File d should now exist in upper but not lower
    let fd = check(open("/mnt/upper/d", OpenFlag::READ_ONLY), "verify d in upper");
    check(close(fd), "close");

    // Overlay umount
    check(overlay_umount("/mnt/merged"), "overlay_umount");

    println!("overlay test passed");
    exit(0);
}

fn check<T>(result: Result<T, SysError>, msg: &str) -> T {
    match result {
        Ok(v) => v,
        Err(e) => {
            print!("{} error={} ", msg, e.as_code());
            exit_with_msg(msg);
        }
    }
}
