use libnet::net_impl::{Read, Write, TcpStream, Duration};

fn usage() -> ! {
    eprintln!("Usage: tcpclient <host> <port> [data]");
    eprintln!("");
    eprintln!("If data is given, sends it and shows response.");
    eprintln!("If no data is given, reads stdin and sends each line.");
    std::process::exit(1);
}

fn do_request(host: &str, port: u16, data: &[u8]) -> Result<(), String> {
    let addr = format!("{}:{}", host, port);
    let mut stream =
        TcpStream::connect(&addr).map_err(|e| format!("connect: {}", e))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

    stream
        .write_all(data)
        .map_err(|e| format!("write: {}", e))?;
    eprintln!("Sent {} bytes", data.len());

    let mut buf = vec![0u8; 65536];
    let mut total = 0usize;
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                    print!("{}", s);
                }
                if n < buf.len() {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(format!("read: {}", e)),
        }
    }
    eprintln!("\nReceived {} bytes", total);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        usage();
    }

    let host = &args[1];
    let port: u16 = args[2].parse().unwrap_or_else(|_| usage());

    if let Some(data) = args.get(3) {
        if let Err(e) = do_request(host, port, data.as_bytes()) {
            eprintln!("tcpclient: {}", e);
            std::process::exit(1);
        }
    } else {
        // Read stdin line by line
        let mut line = String::new();
        loop {
            line.clear();
            let n = std::io::stdin()
                .read_line(&mut line)
                .unwrap_or(0);
            if n == 0 {
                break;
            }
            if let Err(e) = do_request(host, port, line.as_bytes()) {
                eprintln!("tcpclient: {}", e);
                std::process::exit(1);
            }
        }
    }
}
