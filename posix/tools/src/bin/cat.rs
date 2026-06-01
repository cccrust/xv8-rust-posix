use std::io::{self, BufRead, Write};
use std::fs::File;
use std::path::Path;

fn cat_file(path: &Path, number: bool, number_nonblank: bool, squeeze: bool) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut line_no = 1u32;
    let mut prev_blank = false;

    for line in reader.lines() {
        let line = line?;
        let is_blank = line.trim().is_empty();

        if squeeze && is_blank {
            if prev_blank { continue; }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        if number || (number_nonblank && !is_blank) {
            write!(out, "{:>6}\t", line_no)?;
            line_no += 1;
        }
        writeln!(out, "{}", line)?;
    }
    Ok(())
}

fn cat_stdin(number: bool, number_nonblank: bool, squeeze: bool) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut line_no = 1u32;
    let mut prev_blank = false;

    for line in reader.lines() {
        let line = line?;
        let is_blank = line.trim().is_empty();

        if squeeze && is_blank {
            if prev_blank { continue; }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        if number || (number_nonblank && !is_blank) {
            write!(out, "{:>6}\t", line_no)?;
            line_no += 1;
        }
        writeln!(out, "{}", line)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut number = false;
    let mut number_nonblank = false;
    let mut squeeze = false;
    let mut files: Vec<String> = Vec::new();
    let mut i = 1;

    while i < args.len() {
        if args[i] == "--" { i += 1; break; }
        if args[i].starts_with('-') && args[i].len() > 1 {
            for c in args[i][1..].chars() {
                match c {
                    'n' => number = true,
                    'b' => number_nonblank = true,
                    's' => squeeze = true,
                    _ => { eprintln!("cat: invalid option -- '{}'", c); std::process::exit(1); }
                }
            }
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    while i < args.len() {
        files.push(args[i].clone());
        i += 1;
    }

    if files.is_empty() {
        if let Err(e) = cat_stdin(number, number_nonblank, squeeze) {
            eprintln!("cat: stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        for fname in &files {
            if fname == "-" {
                if let Err(e) = cat_stdin(number, number_nonblank, squeeze) {
                    eprintln!("cat: stdin: {}", e);
                    std::process::exit(1);
                }
            } else if let Err(e) = cat_file(Path::new(fname), number, number_nonblank, squeeze) {
                eprintln!("cat: {}: {}", fname, e);
                std::process::exit(1);
            }
        }
    }
}
