#![no_std]
#![no_main]

use user::*;

fn test_getrandom_basic() {
    let mut buf = [0u8; 32];
    let n = raw::getrandom(buf.as_mut_ptr(), buf.len(), 0);
    assert!(n >= 0, "getrandom failed: {n}");
    assert_eq!(n as usize, buf.len(), "getrandom short read");
    let mut non_zero = false;
    for &b in &buf {
        if b != 0 {
            non_zero = true;
            break;
        }
    }
    assert!(non_zero, "getrandom returned all zeros");
    println!("ok getrandom_basic");
}

fn test_getrandom_zero_len() {
    let n = raw::getrandom(core::ptr::null_mut(), 0, 0);
    assert_eq!(n, 0, "getrandom(0) should return 0, got {n}");
    println!("ok getrandom_zero_len");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_getrandom_basic();
    test_getrandom_zero_len();
}
