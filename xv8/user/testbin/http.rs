#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("http: testing minimal HTTP server...");

    let pid = fork().expect("fork");
    if pid == 0 {
        // Child: replace with axum_smoke HTTP server
        exec("/axum_smoke", &["axum_smoke"]);
        unreachable!("exec axum_smoke failed");
    }

    // Parent: wait for server to start
    sleep(100).expect("sleep");

    // Connect to HTTP server
    let sock = tcp_socket().expect("tcp socket");
    tcp_connect(sock, &[127, 0, 0, 1], 27003).expect("tcp connect");
    println!("http: connected");

    // Send HTTP GET request
    let request = b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    let n = tcp_send(sock, request).expect("tcp send");
    println!("http: sent {} bytes", n);

    // Read response
    let mut buf = [0u8; 1024];
    let n = tcp_recv(sock, &mut buf).expect("tcp recv");
    println!("http: recv {} bytes", n);
    let response = core::str::from_utf8(&buf[..n]).unwrap_or("?");
    println!("http: response: {}", response);

    // Check response contains expected content
    assert!(
        response.contains("200 OK"),
        "response must contain 200 OK, got: {}",
        response,
    );
    assert!(
        response.contains("ok"),
        "response body must contain ok, got: {}",
        response,
    );

    close(sock).expect("close sock");

    // Kill the server and wait for it
    kill(pid).expect("kill");
    let mut code = 0;
    wait(&mut code).expect("wait");

    println!("http: PASS");
}
