#![no_std]
#![no_main]

use user::*;

fn run_shell_test() {
    println!("shtest: forking to run shell test script ...");

    let pid = fork().expect("fork");
    if pid == 0 {
        // child: exec the shell with the test script
        exec("/sh", &["sh", "/shtest.sh"]);
        println!("exec failed!");
        exit(1);
    }

    let mut code = 0;
    wait(&mut code).expect("wait");

    if code == 0 {
        println!("shtest: all shell tests passed");
    } else {
        println!("shtest: shell tests FAILED (exit code={})", code);
    }

    exit(code);
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    run_shell_test();
}
