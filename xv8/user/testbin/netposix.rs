#![no_std]
#![no_main]

use kernel::abi::{SockAddrIn, AF_INET, SOCK_STREAM};
use user::*;

fn make_addr(ip: [u8; 4], port: u16) -> SockAddrIn {
    SockAddrIn {
        sin_family: AF_INET,
        sin_port: port.to_be(),
        sin_addr: ip,
        sin_zero: [0u8; 8],
    }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("netposix: test POSIX socket API...");

    // Test 1: socket + bind + listen
    let srv = posix_socket(AF_INET, SOCK_STREAM, 0).expect("srv socket");
    println!("netposix: srv fd={}", srv.as_raw());
    let srv_addr = make_addr([127, 0, 0, 1], 27004);
    posix_bind(srv, &srv_addr).expect("srv bind");
    posix_listen(srv, 1).expect("srv listen");
    println!("netposix: listening on 127.0.0.1:27004");

    // Test 2: connect
    let cli = posix_socket(AF_INET, SOCK_STREAM, 0).expect("cli socket");
    let cli_addr = make_addr([127, 0, 0, 1], 27004);
    posix_connect(cli, &cli_addr).expect("cli connect");
    println!("netposix: connected!");

    // Test 3: accept
    let mut peer_addr = SockAddrIn {
        sin_family: 0,
        sin_port: 0,
        sin_addr: [0u8; 4],
        sin_zero: [0u8; 8],
    };
    let client = posix_accept(srv, &mut peer_addr).expect("srv accept");
    println!("netposix: accepted!");

    // Test 4: send + recv
    let data = b"hello from posix socket!";
    let n = posix_send(cli, data, 0).expect("cli send");
    println!("netposix: sent {} bytes", n);
    assert_eq!(n, data.len(), "sent all data");

    let mut buf = [0u8; 1024];
    let n = posix_recv(client, &mut buf, 0).expect("srv recv");
    println!("netposix: srv recv {} bytes: {}", n, core::str::from_utf8(&buf[..n]).unwrap_or("?"));
    assert_eq!(&buf[..n], data, "data match");

    // Test 5: echo back
    let n = posix_send(client, &buf[..n], 0).expect("srv send");
    let mut echo_buf = [0u8; 1024];
    let n = posix_recv(cli, &mut echo_buf, 0).expect("cli recv");
    assert_eq!(&echo_buf[..n], data, "echo match");

    close(client).expect("close client");
    close(srv).expect("close srv");
    close(cli).expect("close cli");

    println!("netposix: PASS");
}
