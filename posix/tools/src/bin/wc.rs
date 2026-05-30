use std::fs::File;
use std::io::{self, BufRead, BufReader};

struct Counts {
    lines: usize,
    words: usize,
    chars: usize,
    bytes: usize,
}

fn count(reader: &mut impl BufRead) -> Counts {
    let mut c = Counts { lines: 0, words: 0, chars: 0, bytes: 0 };
    let mut buf = String::new();
    loop {
        buf.clear();
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                c.lines += 1;
                c.bytes += buf.len();
                c.chars += buf.chars().count();
                c.words += buf.split_whitespace().count();
            }
            Err(_) => break,
        }
    }
    c
}

fn print_counts(c: &Counts, filename: &str, show_lines: bool, show_words: bool, show_chars: bool, show_bytes: bool) {
    if show_lines { print!("{:>8} ", c.lines); }
    if show_words { print!("{:>8} ", c.words); }
    if show_chars { print!("{:>8} ", c.chars); }
    if show_bytes { print!("{:>8} ", c.bytes); }
    if !filename.is_empty() {
        println!("{}", filename);
    } else {
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut show_lines = true;
    let mut show_words = true;
    let mut show_chars = false;
    let mut show_bytes = true;
    let mut opt_i = 1;

    // Parse options
    while opt_i < args.len() && args[opt_i].starts_with('-') && args[opt_i] != "--" {
        let opt = &args[opt_i];
        if opt == "-l" {
            show_lines = true; show_words = false; show_bytes = false;
        } else if opt == "-w" {
            show_lines = false; show_words = true; show_bytes = false;
        } else if opt == "-c" {
            show_bytes = true; show_lines = false; show_words = false;
        } else if opt == "-m" {
            show_chars = true; show_lines = false; show_words = false; show_bytes = false;
        } else {
            eprintln!("wc: invalid option -- '{}'", &opt[1..]);
            std::process::exit(1);
        }
        opt_i += 1;
    }

    if opt_i == args.len() || (opt_i < args.len() && args[opt_i] == "-") {
        // Read from stdin
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let c = count(&mut reader);
        print_counts(&c, "", show_lines, show_words, show_chars, show_bytes);
    } else {
        let mut total = Counts { lines: 0, words: 0, chars: 0, bytes: 0 };
        let mut filenames: Vec<String> = Vec::new();
        for path in &args[opt_i..] {
            if path == "-" {
                let stdin = io::stdin();
                let mut reader = stdin.lock();
                let c = count(&mut reader);
                print_counts(&c, "", show_lines, show_words, show_chars, show_bytes);
                total.lines += c.lines;
                total.words += c.words;
                total.chars += c.chars;
                total.bytes += c.bytes;
                filenames.push(String::new());
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        let c = count(&mut reader);
                        print_counts(&c, path, show_lines, show_words, show_chars, show_bytes);
                        total.lines += c.lines;
                        total.words += c.words;
                        total.chars += c.chars;
                        total.bytes += c.bytes;
                        filenames.push(path.clone());
                    }
                    Err(e) => {
                        eprintln!("wc: {}: {}", path, e);
                        std::process::exit(1);
                    }
                }
            }
        }
        if filenames.len() > 1 {
            print_counts(&total, "total", show_lines, show_words, show_chars, show_bytes);
        }
    }
}
