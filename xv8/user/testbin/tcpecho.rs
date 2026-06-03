#![no_std]
#![no_main]

use user::*;

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("tcpecho: test TCP syscalls...");

    // Test 1: open and close a TCP socket
    let srv = tcp_socket().expect("srv socket");
    println!("tcpecho: srv fd={}", srv.as_raw());
    close(srv).expect("close");

    // Test 2: open, bind, listen
    let srv = tcp_socket().expect("srv socket");
    tcp_bind(srv, 27000).expect("srv bind");
    tcp_listen(srv).expect("srv listen");

    // Test 3: open a client socket and connect to loopback
    let cli = tcp_socket().expect("cli socket");
    println!("tcpecho: connecting...");
    tcp_connect(cli, &Ipv4Addr::LOOPBACK.0, 27000).expect("cli connect");
    println!("tcpecho: connected!");

    // Test 4: accept the connection
    let client = tcp_accept(srv).expect("srv accept");
    println!("tcpecho: accepted!");

    // Test 5: send and receive data
    let data = b"hello!";
    let n = tcp_send(cli, data).expect("cli send");
    println!("tcpecho: sent {} bytes", n);

    let mut buf = [0u8; 1024];
    let n = tcp_recv(client, &mut buf).expect("srv recv");
    println!("tcpecho: srv recv {} bytes: {}", n, core::str::from_utf8(&buf[..n]).unwrap_or("?"));
    assert_eq!(&buf[..n], data, "data match");

    close(client).expect("close client");
    close(srv).expect("close srv");
    close(cli).expect("close cli");

    println!("tcpecho: PASS");
}