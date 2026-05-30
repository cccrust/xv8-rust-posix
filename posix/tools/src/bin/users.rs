use std::io::{self, BufRead};

fn main() {
    let mut seen = std::collections::BTreeSet::new();
    if let Ok(file) = std::fs::File::open("/var/run/utmpx") {
        let reader = io::BufReader::new(file);
        let _buf = vec![0u8; 384];
        let mut lines = reader.lines();
        while let Some(Ok(line)) = lines.next() {
            if line.len() > 4 {
                let trimmed = line.trim();
                let user = trimmed.split(' ').next().unwrap_or(trimmed);
                if !user.is_empty() && user != " " {
                    seen.insert(user.to_string());
                }
            }
        }
    }
    // Fallback: try `who` output
    if seen.is_empty() {
        if let Ok(output) = std::process::Command::new("who").output() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if let Some(user) = line.split_whitespace().next() {
                    seen.insert(user.to_string());
                }
            }
        }
    }
    let v: Vec<&str> = seen.iter().map(|s| s.as_str()).collect();
    println!("{}", v.join(" "));
}
