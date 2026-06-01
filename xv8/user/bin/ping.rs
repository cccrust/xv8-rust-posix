#![no_std]
#![no_main]

use user::*;
use libnet::proto::icmp::{self, IcmpEchoReply};
use xv8_user_std::net::PingSocket;
use xv8_user_std::process::{uptime, sleep};

const PING_DATA: &[u8] = b"abcdefghijklmnopqrstuvwabcdefghi";
const DEFAULT_COUNT: usize = 4;
const TIMEOUT_TICKS: usize = 100;

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 2 {
        eprintln!("usage: ping <host> [count]");
        exit(1);
    }

    let dest_str = args.get_str(1).unwrap();
    let dest_ip = match dest_str.parse::<Ipv4Addr>() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("invalid host: {}", dest_str);
            exit(1);
        }
    };

    let count = if args.len() >= 3 {
        match args.get_str(2).unwrap().parse::<usize>() {
            Ok(n) => n,
            Err(_) => DEFAULT_COUNT,
        }
    } else {
        DEFAULT_COUNT
    };

    let socket = PingSocket::open().unwrap_or_else(|_| {
        eprintln!("pingsocket failed");
        exit(1);
    });

    println!("PING {} {} bytes of data", dest_ip, PING_DATA.len());

    let mut sent = 0usize;
    let mut recv = 0usize;

    for i in 0..count {
        let id = 0x1234;
        let seq = i as u16;
        let request = icmp::build_echo_request(id, seq, PING_DATA);
        let t0 = uptime();
        if socket.send(&request, &dest_ip.0).is_err() {
            eprintln!("ping: send failed");
            continue;
        }
        sent += 1;

        let mut buf = [0u8; 128];
        let mut got_reply = false;
        let mut waited = 0;
        while waited < TIMEOUT_TICKS {
            if let Ok((len, _src_ip)) = socket.recv(&mut buf) {
                let t1 = uptime();
                let rtt = (t1 - t0) * 10; // convert ticks to ms (each tick = 10ms)
                if let Some(_reply) = icmp::parse_echo_reply(&buf[..len]) {
                    println!("{} bytes from {}: icmp_seq={} time={}ms", PING_DATA.len(), dest_ip, i + 1, rtt);
                    recv += 1;
                    got_reply = true;
                }
                break;
            }
            let _ = sleep(1);
            waited += 1;
        }

        if !got_reply {
            println!("seq={} timeout", i + 1);
        }

        if i + 1 < count {
            let _ = sleep(10);
        }
    }

    // PingSocket closes automatically on drop (no explicit close needed)

    let loss = if sent > 0 { ((sent - recv) * 100) / sent } else { 100 };
    println!("\n--- {} ping statistics ---", dest_ip);
    println!("{} packets transmitted, {} received, {}% packet loss", sent, recv, loss);
}
