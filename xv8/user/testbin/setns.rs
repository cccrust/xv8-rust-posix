#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let pid = raw::getpid() as usize;
    let ns_fd = match raw::nsopen(pid, 5) {
        ret if ret >= 0 => ret,
        ret => {
            println!("FAIL: nsopen returned error {:?}", SysError::from_code((-ret) as u16));
            exit(1);
        }
    };
    match raw::setns(ns_fd as _, CLONE_NEWPID as u32) {
        ret if ret >= 0 => {
            println!("setns test passed");
        }
        ret => {
            let err = SysError::from_code((-ret) as u16);
            println!("FAIL: setns returned unexpected error {:?}", err);
            exit(1);
        }
    }
}
