use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut min_len = 4;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-n" if i + 1 < args.len() => { min_len = args[i + 1].parse().unwrap_or(4); i += 2; }
            _ => { i += 1; }
        }
    }
    let files: Vec<&str> = if i < args.len() { args[i..].iter().map(|s| s.as_str()).collect() } else { vec!["-"] };
    for f in files {
        let content = read_all(f);
        let mut pos = 0;
        let bytes = content.as_bytes();
        while pos < bytes.len() {
            if bytes[pos].is_ascii_graphic() || bytes[pos] == b' ' || bytes[pos] == b'\t' {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_graphic() || bytes[pos] == b' ' || bytes[pos] == b'\t') {
                    pos += 1;
                }
                let run_len = pos - start;
                if run_len as usize >= min_len {
                    let s = String::from_utf8_lossy(&bytes[start..pos]);
                    println!("{}", s);
                }
            } else {
                pos += 1;
            }
        }
    }
}

fn read_all(path: &str) -> String {
    if path == "-" {
        let mut buf = String::new();
        io::stdin().lock().read_to_string(&mut buf).ok();
        buf
    } else {
        std::fs::read_to_string(path).unwrap_or_default()
    }
}
