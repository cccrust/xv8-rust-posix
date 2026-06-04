#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("nettools: test tcpclient+tcpserver...");

    let server_pid = fork().expect("fork server");
    if server_pid == 0 {
        exec("/tcpserver", &["tcpserver", "27999"]);
        unreachable!("exec tcpserver failed");
    }

    let _ = nanosleep(0, 500_000_000);

    let client_pid = fork().expect("fork client");
    if client_pid == 0 {
        exec("/tcpclient", &["tcpclient", "127.0.0.1", "27999", "hello!"]);
        unreachable!("exec tcpclient failed");
    }

    let mut status = 0;
    let _ = wait(&mut status);
    let client_ok = status == 0;

    let _ = kill(server_pid);
    let _ = wait(&mut status);

    if client_ok {
        println!("nettools: PASS");
        exit(0);
    } else {
        println!("nettools: FAILED (client exit={})", status);
        exit(1);
    }
}
