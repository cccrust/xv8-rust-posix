#![no_std]
#![no_main]

use user::*;

const MAX_HOPS: u32 = 30;
const BASE_PORT: u16 = 33434;
const _TIMEOUT_TICKS: usize = 50; // 500ms assuming 10ms ticks
const MAX_BUFFER_SIZE: usize = 512;

/// Simple IPv4 address and port for destination.
#[derive(Debug, Clone, Copy)]
struct SocketAddr {
    ip: [u8; 4],
    port: u16,
}

impl SocketAddr {
    fn new(ip: [u8; 4], port: u16) -> Self {
        Self { ip, port }
    }
}

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 2 {
        eprintln!("usage: traceroute <ip> [max_hops] [timeout_ms]");
        eprintln!("  <ip>: destination IPv4 address");
        eprintln!("  [max_hops]: maximum number of hops (default: 30)");
        eprintln!("  [timeout_ms]: timeout in milliseconds (default: 5000)");
        exit(1);
    }

    let ip_str = args.get_str(1).unwrap();
    let dest_ip = match ip_str.parse::<Ipv4Addr>() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("invalid IP address: {}", ip_str);
            exit(1);
        }
    };

    let max_hops = if args.len() >= 3 {
        match args.get_str(2).unwrap().parse::<u32>() {
            Ok(n) => n,
            Err(_) => MAX_HOPS,
        }
    } else {
        MAX_HOPS
    };

    let timeout_ms = if args.len() >= 4 {
        match args.get_str(3).unwrap().parse::<u64>() {
            Ok(n) => n,
            Err(_) => 5000,
        }
    } else {
        5000
    };

    // Convert timeout_ms to ticks (assuming 10ms per tick)
    let timeout_ticks = (timeout_ms / 10).max(1) as usize;

    println!("traceroute to {}", dest_ip);

    // Create UDP socket for sending packets (port 0 means ephemeral port)
    let socket_fd = match socket(0) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("failed to create socket: {}", e);
            exit(1);
        }
    };

    for ttl in 1..=max_hops {
        // Note: The xv8 kernel's UDP socket does not support setting TTL via setsockopt.
        // We cannot set the TTL in the IP header because we don't have raw IP access.
        // This is a limitation of the current network stack in xv8.
        // We'll just send the packet and rely on the network to decrement TTL (if supported by the underlying hardware/QEMU).

        // Send UDP packet to destination with current TTL (TTL setting is not applied, but we send anyway)
        let addr = SocketAddr::new(dest_ip.0, BASE_PORT + ttl as u16 - 1);
        match send(socket_fd, &[0u8; 0], &addr.ip, addr.port) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("failed to send packet: {}", e);
                continue;
            }
        }

        // Wait for response
        let mut buf = [0u8; MAX_BUFFER_SIZE];
        let mut src_ip = [0u8; 4];
        let mut src_port: u16 = 0;
        let start = uptime();
        let mut got_reply = false;
        let mut waited = 0;

        while waited < timeout_ticks {
            match receive(socket_fd, &mut buf, &mut src_ip, &mut src_port) {
                Ok(_len) => {
                    // We received a packet
                    let elapsed = (uptime() - start) * 10; // convert ticks to ms
                    println!("{:<2} {}.{}.{}.{}  {}ms", ttl, src_ip[0], src_ip[1], src_ip[2], src_ip[3], elapsed);
                    got_reply = true;
                    break;
                }
                Err(_) => {
                    // Timeout or error, continue waiting
                    // Note: We treat any error as timeout for simplicity.
                }
            }
            let _ = sleep(1);
            waited += 1;
        }

        if !got_reply {
            println!("{:<2} *", ttl);
        }

        // If we reached the destination, break
        // Note: Without TTL setting, we cannot know when we reach the destination.
        // We'll break after max_hops anyway.
    }

    // Close the socket (not strictly necessary as the process will exit, but good practice)
    let _ = close(socket_fd);
}