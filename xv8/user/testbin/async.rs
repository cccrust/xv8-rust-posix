#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("async: testing xv8-async TCP echo server...");

    let pid = fork().expect("fork");
    if pid == 0 {
        // Child: replace with async_echo server
        exec("/async_echo", &["async_echo"]);
        unreachable!("exec async_echo failed");
    }

    // Parent: wait for server to start
    sleep(100).expect("sleep");

    // Connect to async_echo server
    let sock = tcp_socket().expect("tcp socket");
    tcp_connect(sock, &[127, 0, 0, 1], 27001).expect("tcp connect");
    println!("async: connected");

    // Send test data
    let data = b"hello from async test!";
    let n = tcp_send(sock, data).expect("tcp send");
    println!("async: sent {} bytes", n);
    assert_eq!(n, data.len(), "sent all data");

    // Receive echo
    let mut buf = [0u8; 1024];
    let n = tcp_recv(sock, &mut buf).expect("tcp recv");
    println!("async: recv {} bytes: {}", n, core::str::from_utf8(&buf[..n]).unwrap_or("?"));
    assert_eq!(&buf[..n], data, "echo must match sent data");

    close(sock).expect("close sock");

    // Kill the server and wait for it
    kill(pid).expect("kill");
    let mut code = 0;
    wait(&mut code).expect("wait");

    println!("async: PASS");
}
