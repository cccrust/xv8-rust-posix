use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: gencat catfile msgfile...");
        std::process::exit(1);
    }
    let output = &args[1];
    let mut catalog: HashMap<u32, HashMap<u32, String>> = HashMap::new();
    for input in &args[2..] {
        let file = match fs::File::open(input) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("gencat: {}: {}", input, e);
                std::process::exit(1);
            }
        };
        let mut set = 1;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if trimmed.starts_with('$') {
                if trimmed.starts_with("$set") {
                    if let Ok(n) = trimmed[4..].trim().parse::<u32>() {
                        set = n;
                    }
                }
                continue;
            }
            if trimmed.is_empty() || trimmed.starts_with('*') || trimmed.starts_with(';') {
                continue;
            }
            if let Some((num, msg)) = trimmed.split_once(char::is_whitespace) {
                if let Ok(n) = num.parse::<u32>() {
                    catalog.entry(set).or_default().insert(n, msg.trim().to_string());
                }
            }
        }
    }
    let mut data = Vec::new();
    for (set, msgs) in &catalog {
        for (num, msg) in msgs {
            data.extend_from_slice(&set.to_le_bytes());
            data.extend_from_slice(&num.to_le_bytes());
            let msg_bytes = msg.as_bytes();
            let len = msg_bytes.len() as u32;
            data.extend_from_slice(&len.to_le_bytes());
            data.extend_from_slice(msg_bytes);
        }
    }
    if let Err(e) = fs::write(output, &data) {
        eprintln!("gencat: {}: {}", output, e);
        std::process::exit(1);
    }
}
