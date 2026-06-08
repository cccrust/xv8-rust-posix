#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("stage: mkdir");
    let _ = mkdir("/myroot");
    let _ = mkdir("/myroot/oldroot");

    println!("stage: open");
    let fd = match open("/myroot", OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => exit_with_msg("open /myroot failed"),
    };
    let _ = close(fd);

    println!("stage: pivot_root");
    match pivot_root("/myroot", "/myroot/oldroot") {
        Ok(()) => println!("stage: ok"),
        Err(e) => {
            println!("pivot_root failed");
            exit_with_msg("pivot_root");
        }
    }

    println!("stage: open oldroot");
    let fd = match open("/oldroot", OpenFlag::READ_ONLY) {
        Ok(fd) => fd,
        Err(_) => exit_with_msg("open /oldroot failed"),
    };
    let _ = close(fd);

    println!("pivot_root test passed");
    exit(0);
}
