use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: csplit <file> <pattern>...");
        std::process::exit(1);
    }
    let file = &args[1];
    let patterns: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
    let lines = read_lines(file);
    let mut chunk = 0;
    let mut start = 0usize;
    let prefix = "xx".to_string();
    let digits = 2;

    for pat in &patterns {
        let mut end = lines.len();
        if let Some(n) = pat.strip_prefix('/') {
            let re = n.trim_end_matches('/');
            for i in start..lines.len() {
                if lines[i].contains(re) {
                    end = i;
                    break;
                }
            }
        } else if let Ok(n) = pat.parse::<usize>() {
            if n > start && n <= lines.len() {
                end = n;
            }
        }
        if end > start {
            let suffix = format!("{:0width$}", chunk, width = digits);
            let filename = format!("{}{}", prefix, suffix);
            let mut f = std::fs::File::create(&filename).unwrap_or_else(|_| {
                eprintln!("csplit: cannot create {}", filename);
                std::process::exit(1);
            });
            for line in &lines[start..end] {
                writeln!(f, "{}", line).ok();
            }
            println!("{}", filename);
            start = end;
            chunk += 1;
        }
    }
    // Write remainder
    if start < lines.len() {
        let suffix = format!("{:0width$}", chunk, width = digits);
        let filename = format!("{}{}", prefix, suffix);
        let mut f = std::fs::File::create(&filename).unwrap_or_else(|_| {
            eprintln!("csplit: cannot create {}", filename);
            std::process::exit(1);
        });
        for line in &lines[start..] {
            writeln!(f, "{}", line).ok();
        }
        println!("{}", filename);
    }
}

fn read_lines(path: &str) -> Vec<String> {
    if path == "-" {
        io::stdin().lock().lines().filter_map(|l| l.ok()).collect()
    } else {
        std::fs::read_to_string(path).unwrap_or_default().lines().map(|l| l.to_string()).collect()
    }
}
