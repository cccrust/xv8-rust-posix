use libnet::dns;

fn dns_lookup(server: &str, domain: &str) -> Result<(), String> {
    let (_header, records) = dns::query(server, domain, dns::TYPE_A)?;
    println!("{} has address", domain);
    for rec in &records {
        if let Some(ip) = rec.to_ipv4() {
            println!("  {}", ip.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("."));
        }
    }
    let (_header, aaaa_records) = dns::query(server, domain, dns::TYPE_AAAA)?;
    for rec in &aaaa_records {
        if let Some(ip) = rec.to_ipv6() {
            println!("  {} has IPv6 address {}",
                domain,
                ip.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().chunks(2).map(|c| c.join("")).collect::<Vec<_>>().join(":")
            );
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: host <domain> [dns-server]");
        std::process::exit(1);
    }

    let domain = &args[1];
    let dns_server = args.get(2).map(|s| s.as_str()).unwrap_or("8.8.8.8");

    if let Err(e) = dns_lookup(dns_server, domain) {
        eprintln!("host: {}: {}", domain, e);
        std::process::exit(1);
    }
}
