#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("std: testing xv8-user-std features...");

    let pid = fork().expect("fork");
    if pid == 0 {
        // Child: replace with test_std tool
        exec("/test_std", &["test_std"]);
        unreachable!("exec test_std failed");
    }

    // Parent: wait for tool to finish
    let mut code = 0;
    wait(&mut code).expect("wait");
    println!("std: child exit code={}", code);

    if code == 0 {
        println!("std: PASS");
    } else {
        println!("std: FAILED");
        exit(1);
    }
}
