use libnet::net_impl::{Read, Write, TcpListener, TcpStream};

fn usage() -> ! {
    eprintln!("Usage: httpd <port>");
    eprintln!("");
    eprintln!("Simple HTTP server that responds with 'hello world' to every request.");
    std::process::exit(1);
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0u8; 1024];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };
    if n == 0 {
        return;
    }

    let body = b"<html><body><h1>hello from xv8!</h1></body></html>\n";
    let status = "HTTP/1.0 200 OK\r\n";
    let content_type = "Content-Type: text/html\r\n";
    let content_len = format!("Content-Length: {}\r\n", body.len());
    let conn_close = "Connection: close\r\n";
    let crlf = "\r\n";

    let mut response = Vec::new();
    response.extend_from_slice(status.as_bytes());
    response.extend_from_slice(content_type.as_bytes());
    response.extend_from_slice(content_len.as_bytes());
    response.extend_from_slice(conn_close.as_bytes());
    response.extend_from_slice(crlf.as_bytes());
    response.extend_from_slice(body);

    let _ = stream.write_all(&response);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let port: u16 = args[1].parse().unwrap_or_else(|_| usage());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("bind: {}", e);
        std::process::exit(1);
    });

    eprintln!("httpd: listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
                break;
            }
            Err(e) => {
                eprintln!("accept error: {}", e);
                break;
            }
        }
    }
}
