use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut tabs = 8;
    let mut initial = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                't' => initial = true,
                'i' => { i += 1; if i < args.len() { tabs = args[i].parse().unwrap_or(8); } }
                _ => { eprintln!("expand: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("expand: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(io::stdin().lock())
    };

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        if initial {
            // Only expand leading tabs
            let mut out = String::new();
            let mut col = 0;
            for c in line.chars() {
                if c == '\t' {
                    let spaces = tabs - (col % tabs);
                    out.push_str(&" ".repeat(spaces));
                    col += spaces;
                } else {
                    out.push(c);
                    col += 1;
                }
            }
            println!("{}", out);
        } else {
            let mut out = String::new();
            for c in line.chars() {
                if c == '\t' {
                    let spaces = tabs - (out.len() % tabs);
                    out.push_str(&" ".repeat(spaces));
                } else {
                    out.push(c);
                }
            }
            println!("{}", out);
        }
    }
}
