use libnet::icmp;
use libnet::util;
use libnet::net_impl::Ipv4Addr;
use libnet::net_impl::Duration;

fn usage() -> ! {
    eprintln!("Usage: ping <ip>");
    std::process::exit(1);
}

fn parse_ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap_or_else(|_| usage())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let target = parse_ip(&args[1]);
    let id = fastrand::u16(..);
    let mut seq = 0u16;
    let mut sent = 0u64;
    let mut received = 0u64;
    let mut total_time = 0.0f64;

    println!("PING {}: 56 data bytes", target);

    for _ in 0..4 {
        match icmp::ping(target, id, seq, Duration::from_secs(2)) {
            Ok((dur, _len)) => {
                let ms = dur.as_secs_f64() * 1000.0;
                println!(
                    "64 bytes from {}: icmp_seq={} time={:.1} ms",
                    target, seq, ms
                );
                received += 1;
                total_time += dur.as_secs_f64();
            }
            Err(e) => {
                eprintln!("Request timeout for icmp_seq={}: {}", seq, e);
            }
        }
        sent += 1;
        seq += 1;
        std::thread::sleep(Duration::from_secs(1));
    }

    let loss = ((sent - received) as f64 / sent as f64) * 100.0;
    let avg = if received > 0 {
        total_time / received as f64
    } else {
        0.0
    };
    println!(
        "\n--- {} ping statistics ---\n\
         {} packets transmitted, {} received, {:.0}% packet loss\n\
         rtt avg = {}",
        target,
        sent,
        received,
        loss,
        util::fmt_duration_us(avg),
    );
}
