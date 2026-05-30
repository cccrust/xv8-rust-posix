use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut width: usize = 75;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'w' => { i += 1; if i < args.len() { width = args[i].parse().unwrap_or(75); } }
                _ => { eprintln!("fmt: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("fmt: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(io::stdin().lock())
    };

    let mut paragraph = String::new();
    for line in reader.lines() {
        let line = line.unwrap_or_default();
        if line.trim().is_empty() {
            if !paragraph.is_empty() {
                print_paragraph(&paragraph, width);
                paragraph.clear();
            }
            println!();
        } else {
            if !paragraph.is_empty() { paragraph.push(' '); }
            paragraph.push_str(line.trim());
        }
    }
    if !paragraph.is_empty() {
        print_paragraph(&paragraph, width);
    }
}

fn print_paragraph(text: &str, width: usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut line = String::new();
    for word in words {
        if line.len() + word.len() + 1 > width && !line.is_empty() {
            println!("{}", line);
            line.clear();
        }
        if !line.is_empty() { line.push(' '); }
        line.push_str(word);
    }
    if !line.is_empty() {
        println!("{}", line);
    }
}
