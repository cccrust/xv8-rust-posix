#![no_std]
#![no_main]

use user::*;

fn test_close_range() {
    let (r1, w1) = pipe2(0).expect("pipe1");
    let (r2, w2) = pipe2(0).expect("pipe2");
    let (r3, w3) = pipe2(0).expect("pipe3");
    let _r4 = pipe2(0).expect("pipe4"); // keep this one open to test non-contiguous range

    let ret = raw::close_range(w1.as_raw(), w3.as_raw(), 0);
    assert_eq!(ret, 0, "close_range failed: {ret}");

    // Writing to closed write ends should fail
    assert!(
        write(w1, b"x").is_err(),
        "write to closed fd should fail"
    );
    assert!(
        write(w2, b"x").is_err(),
        "write to closed fd should fail"
    );
    assert!(
        write(w3, b"x").is_err(),
        "write to closed fd should fail"
    );

    close(r1).expect("close r1");
    close(r2).expect("close r2");
    close(r3).expect("close r3");

    println!("ok close_range");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_close_range();
}
