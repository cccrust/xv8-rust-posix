#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("_http: test httpd+httpget...");

    let server_pid = fork().expect("fork server");
    if server_pid == 0 {
        exec("/httpd", &["httpd", "27998"]);
        unreachable!("exec httpd failed");
    }

    let _ = nanosleep(0, 500_000_000);

    let client_pid = fork().expect("fork client");
    if client_pid == 0 {
        exec("/httpget", &["httpget", "http://127.0.0.1:27998/"]);
        unreachable!("exec httpget failed");
    }

    let mut status = 0;
    let _ = wait(&mut status);

    if status == 0 {
        println!("_http: PASS");
        exit(0);
    } else {
        println!("_http: FAILED (client exit={})", status);
        exit(1);
    }
}
