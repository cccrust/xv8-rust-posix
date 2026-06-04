use libnet::net_impl::{Read, Write, TcpStream, Duration};

fn usage() -> ! {
    eprintln!("Usage: httpget <url>");
    eprintln!("");
    eprintln!("Example: httpget http://example.com/");
    std::process::exit(1);
}

fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "only http:// URLs supported".to_string())?;

    let (host_part, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{}", p)),
        None => (rest, "/".to_string()),
    };

    let (host, port) = if let Some((h, p)) = host_part.split_once(':') {
        let port: u16 = p.parse().map_err(|_| format!("invalid port: {}", p))?;
        (h.to_string(), port)
    } else {
        (host_part.to_string(), 80)
    };

    Ok((host, port, path))
}

fn http_get(url: &str) -> Result<(), String> {
    let (host, port, path) = parse_url(url)?;

    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("connect: {}", e))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));

    let request = format!("GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host);
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("write: {}", e))?;

    let mut buf = vec![0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                    print!("{}", s);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(format!("read: {}", e)),
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    if let Err(e) = http_get(&args[1]) {
        eprintln!("httpget: {}", e);
        std::process::exit(1);
    }
}
