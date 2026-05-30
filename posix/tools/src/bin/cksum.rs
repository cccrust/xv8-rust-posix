use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let files: Vec<&str> = if args.len() > 1 {
        args[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec!["-"]
    };
    for f in files {
        let content = read_all(f);
        let cksum = crc32(&content);
        let size = content.len();
        println!("{} {} {}", cksum, size, if f == "-" { "" } else { f });
    }
}

fn read_all(path: &str) -> Vec<u8> {
    if path == "-" {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf).ok();
        buf
    } else {
        std::fs::read(path).unwrap_or_default()
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}
