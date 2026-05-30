use std::io::{self, BufRead, Write};
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let files: Vec<&str> = if args.len() > 1 {
        args[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec!["-"]
    };
    for file in files {
        if file == "-" {
            page_reader(io::stdin().lock());
        } else {
            match File::open(file) {
                Ok(f) => { page_reader(io::BufReader::new(f)); }
                Err(e) => {
                    eprintln!("more: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn page_reader<R: BufRead>(mut r: R) {
    let stdin = io::stdin();
    let mut line = String::new();
    let mut lines_shown = 0u32;
    let term_height = 24;

    loop {
        line.clear();
        let bytes = r.read_line(&mut line).unwrap_or(0);
        if bytes == 0 { break; }
        print!("{}", line);
        io::stdout().flush().ok();
        lines_shown += 1;

        if lines_shown >= term_height - 1 {
            eprint!("--More--({}%)", percent_done(&r));
            io::stderr().flush().ok();
            let mut input = String::new();
            stdin.read_line(&mut input).ok();
            let input = input.trim();
            match input {
                "q" | "Q" => { break; }
                " " => { lines_shown = 0; }
                "\n" | "" => { lines_shown = term_height - 2; }
                _ => { lines_shown = 0; }
            }
        }
    }
}

fn percent_done<R: BufRead>(_r: &R) -> u32 {
    0
}
