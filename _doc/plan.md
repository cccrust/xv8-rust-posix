# Network Tools Expansion Plan

This document outlines the plan for adding more network tools to the `net/tools` crate.
The goal is to implement common Linux network utilities in Rust that can run on the host machine
(without requiring xv8 or QEMU). Each tool should be a standalone binary in `net/tools/src/bin/`.

## Existing Tools
- `ping.rs` - ICMP echo request
- `host.rs` - DNS lookup (hostname to IP)
- `dns.rs` - DNS query tool
- `whois.rs` - WHOIS client
- `ntp.rs` - Network Time Protocol client
- `tcpclient.rs` - TCP client for testing connections
- `tcpserver.rs` - TCP server for testing connections
- `tftp.rs` - Trivial File Transfer Protocol client

## Planned Tools

### Connection Testing
- `traceroute.rs` - Trace the path packets take to a network host
- `netstat.rs` - Display network connections, routing tables, interface statistics
- `ss.rs` - Socket statistics (modern replacement for netstat)
- `nc.rs` or `netcat.rs` - Arbitrary TCP/UDP connections and listening

### File Transfer
- `ftp.rs` - File Transfer Protocol client
- `scp.rs` - Secure Copy Protocol client (simplified, using SSH)
- `wget.rs` - Network downloader for HTTP/HTTPS
- `curl.rs` - URL transfer utility (supporting multiple protocols)

### Network Configuration
- `ifconfig.rs` or `ip.rs` - Display/configure network interfaces (Linux `ip` command style)
- `route.rs` - Display/configure IP routing table
- `arp.rs` - Display/configure ARP cache
- `iptables.rs` - Configure Linux packet filtering rules (simplified)

### Monitoring & Diagnostics
- `tcpdump.rs` - Network packet analyzer (simplified version)
- `mtr.rs` - Combined traceroute and ping
- `iftop.rs` - Display bandwidth usage on interfaces
- `nmap.rs` - Network scanner (basic port scanning)

### Services
- `ssh.rs` - SSH client (basic functionality)
- `telnet.rs` - Telnet client
- `dhcpcd.rs` - DHCP client daemon (simplified)
- `rsync.rs` - Fast, versatile file copying tool (basic)

## Implementation Guidelines
1. Each tool should be a single binary in `net/tools/src/bin/<toolname>.rs`
2. Use existing Rust crates where possible (e.g., `trust-dns` for DNS, `tokio` for async)
3. For system-specific functionality, use standard library or `libc` crate
4. Tools should mimic the behavior and command-line interface of their Linux counterparts
5. Where full implementation is complex, provide a simplified version that covers common use cases
6. Add unit tests for core functionality where feasible
7. Ensure tools build and run on the host machine (Linux/macOS) without xv8

## Priority
First wave (easier to implement):
- `traceroute.rs` (based on UDP/ICMP echo)
- `netstat.rs` (read from /proc on Linux, use sysctls on BSD/macOS)
- `nc.rs` (simple TCP/UDP client/server)
- `wget.rs` (HTTP GET using reqwest or similar)
- `ifconfig.rs`/ `ip.rs` (interface information)

Second wave:
- `ftp.rs`, `scp.rs`, `ssh.rs` (require handling authentication and encryption)
- `tcpdump.rs` (requires packet capture, may need platform-specific permissions)
- `nmap.rs` (port scanning with service detection)

Note: Some tools may require elevated privileges (e.g., raw sockets for ping/traceroute). 
We will document any permission requirements.

## Integration with shell.sh
Once implemented, these tools will automatically be available in the POSIX shell when running:
```bash
./shell.sh
```
as the script builds and adds the net/tools binaries to PATH.
