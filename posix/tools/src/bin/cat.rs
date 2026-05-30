use std::io::{self, Read, Write};
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // Read from stdin
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut handle = stdin.lock();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        loop {
            match handle.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if out.write_all(&buf[..n]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    } else {
        for path in &args[1..] {
            if path == "-" {
                let stdin = io::stdin();
                let stdout = io::stdout();
                let mut handle = stdin.lock();
                let mut out = stdout.lock();
                let mut buf = [0u8; 8192];
                loop {
                    match handle.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if out.write_all(&buf[..n]).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            } else {
                match File::open(path) {
                    Ok(mut file) => {
                        let stdout = io::stdout();
                        let mut out = stdout.lock();
                        let mut buf = [0u8; 8192];
                        loop {
                            match file.read(&mut buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if out.write_all(&buf[..n]).is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("cat: {}: {}", path, e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("cat: {}: {}", path, e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
