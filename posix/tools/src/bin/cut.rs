use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut delim: u8 = b'\t';
    let mut fields: Vec<usize> = Vec::new();
    let mut chars: Vec<usize> = Vec::new();
    let mut bytes: Vec<usize> = Vec::new();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'd' => { i += 1; if i < args.len() { delim = args[i].as_bytes()[0]; } }
                'f' => { i += 1; if i < args.len() { fields = parse_range(&args[i]); } }
                'c' => { i += 1; if i < args.len() { chars = parse_range(&args[i]); } }
                'b' => { i += 1; if i < args.len() { bytes = parse_range(&args[i]); } }
                _ => { eprintln!("cut: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<String> = args[i..].to_vec();
    let reader: Box<dyn BufRead> = if files.is_empty() {
        Box::new(io::stdin().lock())
    } else {
        Box::new(io::BufReader::new(std::fs::File::open(&files[0]).unwrap_or_else(|e| {
            eprintln!("cut: {}: {}", files[0], e);
            std::process::exit(1);
        })))
    };

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        if !fields.is_empty() {
            let parts: Vec<&str> = if delim == 0 { line.split_whitespace().collect() } else { line.split(|c| c == delim as char).collect() };
            let out: Vec<&str> = fields.iter().filter_map(|&f| {
                if f > 0 && f <= parts.len() { Some(parts[f - 1]) } else { None }
            }).collect();
            println!("{}", out.join(&(delim as char).to_string()));
        } else if !chars.is_empty() || !bytes.is_empty() {
            let indices = if !chars.is_empty() { &chars } else { &bytes };
            let cs: Vec<char> = line.chars().collect();
            let out: String = indices.iter().filter_map(|&f| {
                if f > 0 && f <= cs.len() { Some(cs[f - 1]) } else { None }
            }).collect();
            println!("{}", out);
        } else {
            println!("{}", line);
        }
    }
}

fn parse_range(s: &str) -> Vec<usize> {
    let mut ranges = Vec::new();
    for part in s.split(',') {
        if let Some((start, end)) = part.split_once('-') {
            let lo: usize = start.parse().unwrap_or(1);
            let hi: usize = if end.is_empty() { usize::MAX } else { end.parse().unwrap_or(lo) };
            for v in lo..=hi.min(lo + 1000) {
                ranges.push(v);
            }
        } else {
            if let Ok(n) = part.parse() {
                ranges.push(n);
            }
        }
    }
    ranges
}
