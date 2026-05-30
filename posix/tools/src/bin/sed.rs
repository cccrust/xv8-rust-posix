use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: sed [-n] [-e script] [file ...]");
        std::process::exit(1);
    }

    let script = &args[i];
    i += 1;
    let files: Vec<String> = args[i..].to_vec();

    // Parse: s/old/new/g
    let (pattern, replacement, global) = parse_sub(script);

    for fname in &files {
        let content = std::fs::read_to_string(fname).unwrap_or_else(|e| {
            eprintln!("sed: {}: {}", fname, e);
            std::process::exit(1);
        });
        for line in content.lines() {
            println!("{}", apply(line, &pattern, &replacement, global));
        }
    }

    if files.is_empty() {
        for line in io::stdin().lock().lines() {
            let line = line.unwrap_or_default();
            println!("{}", apply(&line, &pattern, &replacement, global));
        }
    }
}

fn parse_sub(script: &str) -> (String, String, bool) {
    if let Some(rest) = script.strip_prefix('s') {
        let delim = rest.chars().next().unwrap_or('/');
        let parts: Vec<&str> = rest[1..].splitn(3, delim).collect();
        if parts.len() >= 2 {
            let global = parts.get(2).map(|f| f.contains('g')).unwrap_or(false);
            return (parts[0].to_string(), parts[1].to_string(), global);
        }
    }
    (String::new(), String::new(), false)
}

fn apply(line: &str, pattern: &str, replacement: &str, global: bool) -> String {
    if pattern.is_empty() { return line.to_string(); }
    if global {
        line.replace(pattern, replacement)
    } else {
        line.replacen(pattern, replacement, 1)
    }
}
