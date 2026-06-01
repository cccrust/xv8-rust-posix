use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::process::exit;
use std::time::{Duration, Instant};


fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_help();
        exit(0);
    }
    
    let destination = &args[1];
    let max_hops = args.get(2)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(30);
    let timeout_ms = args.get(3)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5000);
    
    match traceroute(destination, max_hops, Duration::from_millis(timeout_ms)) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("traceroute: {}", e);
            exit(1);
        }
    }
}

fn print_help() {
    println!("traceroute - Trace the route packets take to a network host
Usage: traceroute [OPTIONS] <destination> [max_hops] [timeout_ms]

Options:
  -h, --help  Display this help and exit

This is a simplified traceroute implementation using UDP packets.
Note: May require elevated privileges on some systems.");
}

fn traceroute(destination: &str, max_hops: u32, timeout: Duration) -> Result<(), String> {
    // Resolve destination hostname to IP address
    let dest_ip = match destination.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) => {
            // Try DNS resolution using our libnet
            match libnet::dns::query("8.8.8.8", destination, libnet::dns::TYPE_A) {
                Ok((_header, records)) => {
                    if records.is_empty() {
                        return Err(format!("Unknown host {}", destination));
                    }
                    if let Some(ip) = records[0].to_ipv4() {
                        IpAddr::V4(Ipv4Addr::from(ip))
                    } else {
                        return Err(format!("No IPv4 address found for {}", destination));
                    }
                }
                Err(e) => return Err(format!("DNS resolution failed: {}", e)),
            }
        }
    };
    
    println!("traceroute to {} ({}), {} hops max, {} byte packets", 
             destination, dest_ip, max_hops, 0);
    
    // Create UDP socket for sending packets
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to create socket: {}", e))?;
    socket.set_read_timeout(Some(timeout))
        .map_err(|e| format!("Failed to set socket timeout: {}", e))?;
    
    // Port range for traceroute (typically 33434-33534)
    let base_port = 33434u16;
    
    for ttl in 1..=max_hops {
        // Set TTL on socket
        socket.set_ttl(ttl)
            .map_err(|e| format!("Failed to set TTL: {}", e))?;
        
        // Send UDP packet to destination with current TTL
        let addr = SocketAddr::new(dest_ip, base_port + ttl as u16 - 1);
        let _sent = socket.send_to(&[0u8; 0], &addr)
            .map_err(|e| format!("Failed to send packet: {}", e))?;
        
        // Wait for response
        let mut buf = [0u8; 512];
        let start = Instant::now();
        match socket.recv_from(&mut buf) {
            Ok((_size, src)) => {
                let elapsed = start.elapsed();
                println!("{:<2} {}  {:.2?} ms", ttl, src.ip(), elapsed);
                
                // If we reached the destination (ICMP port unreachable or we got a response from target)
                if src.ip() == dest_ip {
                    break;
                }
            }
            Err(_) => {
                // Timeout
                println!("{:<2} *", ttl);
            }
        }
    }
    
    Ok(())
}