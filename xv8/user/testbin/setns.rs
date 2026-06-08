#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    match raw::setns(0, 0) {
        ret if ret < 0 => {
            let err = SysError::from_code((-ret) as u16);
            if err == SysError::NotImplemented {
                println!("setns stub test passed");
            } else {
                println!("FAIL: setns returned unexpected error {:?}", err);
                exit(1);
            }
        }
        _ => {
            println!("FAIL: setns succeeded unexpectedly");
            exit(1);
        }
    }
}
