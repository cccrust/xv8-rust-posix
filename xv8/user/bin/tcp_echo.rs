#![no_std]
#![no_main]

use user::*;

const USAGE: &str = "usage: tcp_echo <port>";

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg(USAGE);
    }

    let port = args.get_str(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| exit_with_msg("invalid port number"));

    let fd = tcp_socket().expect("tcp_socket failed");
    tcp_bind(fd, port).expect("tcp_bind failed");
    tcp_listen(fd).expect("tcp_listen failed");

    let mut buf = [0u8; 4096];

    loop {
        let client = tcp_accept(fd).expect("tcp_accept failed");
        println!("accepted connection on fd={}", client.as_raw());

        let n = tcp_recv(client, &mut buf).expect("tcp_recv failed");
        if n > 0 {
            let _ = tcp_send(client, &buf[..n]);
        }

        close(client).expect("close failed");
    }
}