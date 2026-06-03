#![no_std]
#![no_main]

use user::*;

const TEST_PORT: u16 = 27000;
const DATA: &[u8] = b"hello tcp echo!";

fn server() {
    let srv = tcp_socket().expect("srv socket");
    tcp_bind(srv, TEST_PORT).expect("srv bind");
    tcp_listen(srv).expect("srv listen");

    // Accept one connection
    let client = tcp_accept(srv).expect("srv accept");
    let mut buf = [0u8; 1024];
    let n = tcp_recv(client, &mut buf).expect("srv recv");
    assert_eq!(&buf[..n], DATA, "srv received correct data");

    let n = tcp_send(client, &buf[..n]).expect("srv send");
    assert_eq!(n, DATA.len(), "send echo must return byte count");

    close(client).expect("close client fd");
    close(srv).expect("close srv");
}

fn client() {
    // Small delay to ensure server is listening
    sleep(1);

    let cli = tcp_socket().expect("cli socket");
    tcp_connect(cli, &Ipv4Addr::LOOPBACK.0, TEST_PORT).expect("cli connect");

    let n = tcp_send(cli, DATA).expect("cli send");
    assert_eq!(n, DATA.len(), "send must return byte count");

    let mut rbuf = [0u8; 1024];
    let n = tcp_recv(cli, &mut rbuf).expect("cli recv");
    assert_eq!(&rbuf[..n], DATA, "cli received echo");

    close(cli).expect("close cli");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("tcpecho: testing TCP echo via loopback...");

    if fork().expect("fork") == 0 {
        // Child: client
        client();
    } else {
        // Parent: server
        server();
        let mut status = 0;
        wait(&mut status).expect("wait");
        assert_eq!(status, 0, "child must exit 0");
    }

    println!("tcpecho: PASS");
}