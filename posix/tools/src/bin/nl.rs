use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut number_all = false;
    let mut number_nonempty = true;
    let start = 1usize;
    let inc = 1usize;
    let mut sep = "\t";
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'b' => number_nonempty = false,
                'n' => number_all = false,
                's' => sep = "",
                'v' => number_all = true,
                _ => { eprintln!("nl: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let reader: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("nl: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(io::stdin().lock())
    };

    let mut lineno = start;

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        let is_empty = line.trim().is_empty();
        let should_number = if number_all {
            number_all
        } else if number_nonempty {
            !is_empty
        } else {
            !is_empty
        };

        if should_number {
            println!("{:>6}{}{}", lineno, sep, line);
            lineno += inc;
        } else {
            println!("{:>6}{}{}", "", sep, line);
        }
    }
}
