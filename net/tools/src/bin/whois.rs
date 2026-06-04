use libnet::net_impl::{Read, Write, TcpStream, Duration};

fn usage() -> ! {
    eprintln!("Usage: whois <query> [server]");
    eprintln!("");
    eprintln!("Default server: whois.verisign-grs.com");
    std::process::exit(1);
}

fn whois_lookup(query: &str, server: &str) -> Result<String, String> {
    let addr = format!("{}:43", server);
    let mut stream =
        TcpStream::connect(&addr).map_err(|e| format!("connect: {}", e))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));

    let mut request = query.to_string();
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {}", e))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("read: {}", e))?;

    Ok(response)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let query = &args[1];
    let server = args.get(2).map(|s| s.as_str()).unwrap_or("whois.verisign-grs.com");

    match whois_lookup(query, server) {
        Ok(response) => {
            println!("{}", response.trim());
        }
        Err(e) => {
            eprintln!("whois: {}: {}", query, e);
            std::process::exit(1);
        }
    }
}
