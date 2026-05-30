use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut width: usize = 80;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'w' => { i += 1; if i < args.len() { width = args[i].parse().unwrap_or(80); } }
                _ => { eprintln!("fold: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("fold: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(io::stdin().lock())
    };

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        let mut pos = 0;
        let bytes = line.as_bytes();
        while pos < bytes.len() {
            let end = (pos + width).min(bytes.len());
            // Try to break at a space if possible
            let break_at = if end < bytes.len() {
                bytes[pos..end].iter().rposition(|&b| b == b' ').map(|r| pos + r + 1).unwrap_or(end)
            } else {
                end
            };
            println!("{}", &line[pos..break_at]);
            pos = break_at;
        }
    }
}
