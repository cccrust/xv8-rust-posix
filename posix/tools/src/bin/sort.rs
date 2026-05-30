use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut reverse = false;
    let mut numeric = false;
    let mut unique = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'r' => reverse = true,
                'n' => numeric = true,
                'u' => unique = true,
                _ => { eprintln!("sort: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<String> = args[i..].to_vec();
    let mut lines: Vec<String> = Vec::new();

    if files.is_empty() {
        for line in io::stdin().lock().lines() {
            lines.push(line.unwrap_or_default());
        }
    } else {
        for fname in files {
            let content = std::fs::read_to_string(&fname).unwrap_or_else(|e| {
                eprintln!("sort: {}: {}", &fname, e);
                std::process::exit(1);
            });
            for line in content.lines() {
                lines.push(line.to_string());
            }
        }
    }

    if numeric {
        lines.sort_by(|a, b| {
            let an: f64 = a.trim().parse().unwrap_or(0.0);
            let bn: f64 = b.trim().parse().unwrap_or(0.0);
            an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        lines.sort();
    }

    if reverse {
        lines.reverse();
    }

    let mut prev: Option<String> = None;
    for line in &lines {
        if unique {
            if prev.as_deref() == Some(line.as_str()) { continue; }
            prev = Some(line.clone());
        }
        println!("{}", line);
    }
}
