use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        exit(0);
    }

    #[cfg(target_os = "linux")]
    {
        print_linux_netstat();
    }

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("netstat: This implementation is only available on Linux.");
        eprintln!("On this platform, try using 'ss' or platform-specific tools.");
        exit(1);
    }
}

fn print_help() {
    println!("netstat - Network statistics
Usage: netstat [OPTIONS]

Options:
  -h, --help  Display this help and exit

This is a simplified netstat implementation that shows active TCP connections.
For full functionality, install the net-tools package.");
}

#[cfg(target_os = "linux")]
fn print_linux_netstat() {
    // Read TCP connections from /proc/net/tcp and /proc/net/tcp6
    print_header("TCP");
    print_connections("/proc/net/tcp", false);
    print_connections("/proc/net/tcp6", true);

    // UDP connections
    print_header("UDP");
    print_connections("/proc/net/udp", false);
    print_connections("/proc/net/udp6", true);
}

#[cfg(target_os = "linux")]
fn print_header(proto: &str) {
    println!("{:<6} {:<22} {:<22} {:<6} {:<6}", 
             "Proto", "Local Address", "Foreign Address", "State", "PID/Program");
    println!("{:<6} {:<22} {:<22} {:<6} {:<6}", 
             "-----", "-------------", "--------------", "-----", "---------");
}

#[cfg(target_os = "linux")]
fn print_connections(path: &str, is_ipv6: bool) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // File might not exist or be readable
    };

    let lines: Vec<&str> = content.lines().skip(1).collect(); // Skip header
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        // Format: sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
        let local_addr = parts[1];
        let rem_addr = parts[2];
        let state = parts[3];

        let local_socket = parse_socket_addr(local_addr, is_ipv6);
        let remote_socket = parse_socket_addr(rem_addr, is_ipv6);

        // State mapping for TCP
        let state_str = if state == "01" {
            "ESTAB"
        } else if state == "02" {
            "SYN_SENT"
        } else if state == "03" {
            "SYN_RECV"
        } else if state == "04" {
            "FIN_WAIT1"
        } else if state == "05" {
            "FIN_WAIT2"
        } else if state == "06" {
            "TIME_WAIT"
        } else if state == "07" {
            "CLOSE"
        } else if state == "08" {
            "CLOSE_WAIT"
        } else if state == "09" {
            "LAST_ACK"
        } else if state == "0A" {
            "LISTEN"
        } else if state == "0B" {
            "CLOSING"
        } else {
            "UNKNOWN"
        };

        // For simplicity, we don't show PID/Program in this example
        println!("{:<6} {:<22} {:<22} {:<6} {:<6}", 
                 if is_ipv6 { "TCPv6" } else { "TCP" },
                 format_socket(&local_socket),
                 format_socket(&remote_socket),
                 state_str,
                 "-");
    }
}

#[cfg(target_os = "linux")]
fn parse_socket_addr(addr: &str, is_ipv6: bool) -> std::net::SocketAddr {
    if is_ipv6 {
        // Format: [hex ipv6 address]:[hex port]
        if addr.len() < 34 { // [0-9a-f]{32}:[0-9a-f]{4}
            return std::net::SocketAddr::new(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0);
        }
        let ip_hex = &addr[0..32];
        let port_hex = &addr[33..37]; // Skip the colon at index 32

        let ipv6 = std::net::Ipv6Addr::from(
            u128::from_str_radix(ip_hex, 16).unwrap_or(0)
        );
        let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);
        std::net::SocketAddr::new(std::net::IpAddr::V6(ipv6), port)
    } else {
        // Format: [hex ipv4 address]:[hex port]
        if addr.len() < 12 { // [0-9a-f]{8}:[0-9a-f]{4}
            return std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0);
        }
        let ip_hex = &addr[0..8];
        let port_hex = &addr[9..13]; // Skip the colon at index 8

        let ipv4 = std::net::Ipv4Addr::from(
            u32::from_str_radix(ip_hex, 16).unwrap_or(0)
        );
        let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);
        std::net::SocketAddr::new(std::net::IpAddr::V4(ipv4), port)
    }
}

#[cfg(target_os = "linux")]
fn format_socket(addr: &std::net::SocketAddr) -> String {
    match addr {
        std::net::SocketAddr::V4(ref v4) => format!("{}:{}", v4.ip(), v4.port()),
        std::net::SocketAddr::V6(ref v6) => format!("[{}]:{}", v6.ip(), v6.port()),
    }
}