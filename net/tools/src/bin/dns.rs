use libnet::dns;

fn usage() -> ! {
    eprintln!("Usage: dns <domain> [dns-server]");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let domain = &args[1];
    let server = args.get(2).map(|s| s.as_str()).unwrap_or("8.8.8.8");

    match dns::query(server, domain, dns::TYPE_A) {
        Ok((_header, records)) => {
            println!("Query: {} -> {}", domain, server);
            for rec in &records {
                if let Some(ip) = rec.to_ipv4() {
                    println!("  A  {} (TTL={})",
                        ip.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("."),
                        rec.ttl
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("dns: {}: {}", domain, e);
            std::process::exit(1);
        }
    }
}
