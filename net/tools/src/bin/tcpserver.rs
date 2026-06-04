use libnet::net_impl::{Read, Write, TcpListener, TcpStream, SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "xv8"))]
use std::thread;

enum Mode {
    Echo,
    Daytime,
    Time,
}

fn usage() -> ! {
    eprintln!("Usage: tcpserver <port> [--echo|--daytime|--time]");
    std::process::exit(1);
}

fn format_rfc3339(now: SystemTime) -> String {
    let dur = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let nsecs = dur.subsec_nanos();

    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if d < md {
            m = i + 1;
            break;
        }
        d -= md;
    }
    if m == 0 {
        m = 12;
        d = 0;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, m, day, hours, minutes, seconds, nsecs / 1_000_000
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn handle_client(mut stream: TcpStream, mode: Mode) {
    let peer = stream.peer_addr();
    match mode {
        Mode::Echo => {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Err(e) = stream.write_all(&buf[..n]) {
                            eprintln!("write error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("read error: {}", e);
                        break;
                    }
                }
            }
        }
        Mode::Daytime => {
            let now = SystemTime::now();
            let datetime = format_rfc3339(now);
            let _ = stream.write_all(datetime.as_bytes());
            let _ = stream.write_all(b"\n");
        }
        Mode::Time => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(stream, "{}", now);
        }
    }
    if let Ok(addr) = peer {
        eprintln!("Connection closed: {}", addr);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let port: u16 = args[1].parse().unwrap_or_else(|_| usage());

    let mode = if args.len() > 2 {
        match args[2].as_str() {
            "--echo" => Mode::Echo,
            "--daytime" => Mode::Daytime,
            "--time" => Mode::Time,
            _ => usage(),
        }
    } else {
        Mode::Echo
    };

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("bind: {}", e);
        std::process::exit(1);
    });

    eprintln!("Listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Ok(addr) = stream.peer_addr() {
                    eprintln!("Connection from: {}", addr);
                }
                let mode_clone = match mode {
                    Mode::Echo => Mode::Echo,
                    Mode::Daytime => Mode::Daytime,
                    Mode::Time => Mode::Time,
                };
                #[cfg(not(feature = "xv8"))]
                thread::spawn(move || {
                    handle_client(stream, mode_clone);
                });
                #[cfg(feature = "xv8")]
                handle_client(stream, mode_clone);
            }
            Err(e) => {
                eprintln!("accept error: {}", e);
            }
        }
    }
}
