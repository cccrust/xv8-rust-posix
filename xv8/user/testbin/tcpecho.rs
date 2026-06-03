#![no_std]
#![no_main]

use user::*;

const TEST_PORT: u16 = 27000;
const DATA: &[u8] = b"hello tcp echo!";

fn test_tcp_echo() {
    // Server: listen on TEST_PORT
    let srv = tcp_socket().expect("srv socket");
    tcp_bind(srv, TEST_PORT).expect("srv bind");
    tcp_listen(srv).expect("srv listen");

    // Client: connect to loopback TEST_PORT
    let cli = tcp_socket().expect("cli socket");
    tcp_connect(cli, &Ipv4Addr::LOOPBACK.0, TEST_PORT).expect("cli connect");

    // Server: accept
    let client = tcp_accept(srv).expect("srv accept");

    // Client: send data
    let n = tcp_send(cli, DATA).expect("cli send");
    assert_eq!(n, DATA.len(), "send must return byte count");

    // Server: recv and echo back
    let mut buf = [0u8; 1024];
    let n = tcp_recv(client, &mut buf).expect("srv recv");
    assert_eq!(&buf[..n], DATA, "srv received correct data");

    let n = tcp_send(client, &buf[..n]).expect("srv send");
    assert_eq!(n, DATA.len(), "send echo must return byte count");

    // Client: recv echo
    let mut rbuf = [0u8; 1024];
    let n = tcp_recv(cli, &mut rbuf).expect("cli recv");
    assert_eq!(&rbuf[..n], DATA, "cli received echo");

    close(client).expect("close client fd");
    close(srv).expect("close srv");
    close(cli).expect("close cli");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("tcpecho: testing TCP echo via loopback...");
    test_tcp_echo();
    println!("tcpecho: PASS");
}