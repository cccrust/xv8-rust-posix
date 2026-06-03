use std::io::{self, BufRead};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut prefix = String::new();
    let mut dict_file = "/usr/share/dict/words".to_string();
    let mut i = 1;
    while i < args.len() {
        if !args[i].starts_with('-') && prefix.is_empty() {
            prefix = args[i].clone();
        } else if !args[i].starts_with('-') {
            dict_file = args[i].clone();
        }
        i += 1;
    }

    if prefix.is_empty() {
        eprintln!("Usage: look <prefix> [dictionary_file]");
        std::process::exit(1);
    }

    let content = match fs::read_to_string(&dict_file) {
        Ok(c) => c,
        Err(_) => {
            // Try alternative paths
            let alt_paths = [
                "/usr/share/dict/web2",
                "/usr/local/share/dict/words",
                "/usr/dict/words",
            ];
            let mut found = None;
            for p in &alt_paths {
                if let Ok(c) = fs::read_to_string(p) {
                    found = Some(c);
                    break;
                }
            }
            match found {
                Some(c) => c,
                None => {
                    eprintln!("look: {}: not found", dict_file);
                    std::process::exit(1);
                }
            }
        }
    };

    for line in content.lines() {
        if line.to_lowercase().starts_with(&prefix.to_lowercase()) {
            println!("{}", line);
        }
    }
}