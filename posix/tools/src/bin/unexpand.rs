use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut tabs = 8;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'a' => {} // all (default)
                't' => { i += 1; if i < args.len() { tabs = args[i].parse().unwrap_or(8); } }
                _ => { eprintln!("unexpand: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("unexpand: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(io::stdin().lock())
    };

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        // Convert spaces back to tabs
        let mut out = String::new();
        let mut col = 0;
        let chars: Vec<char> = line.chars().collect();
        let mut j = 0;
        while j < chars.len() {
            if chars[j] == ' ' {
                let mut space_count = 0;
                while j + space_count < chars.len() && chars[j + space_count] == ' ' {
                    space_count += 1;
                }
                let next_tab = tabs - (col % tabs);
                if space_count >= next_tab {
                    out.push('\t');
                    col += next_tab;
                    j += next_tab;
                } else {
                    for _ in 0..space_count {
                        out.push(' ');
                        col += 1;
                    }
                    j += space_count;
                }
            } else {
                out.push(chars[j]);
                col += 1;
                j += 1;
            }
        }
        println!("{}", out);
    }
}
