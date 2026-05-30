use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut append = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'a' => append = true,
                _ => { eprintln!("tee: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<String> = args[i..].to_vec();
    let mut writers: Vec<Box<dyn Write>> = Vec::new();

    for fname in &files {
        let file = if append {
            File::options().append(true).create(true).open(Path::new(fname))
        } else {
            File::create(Path::new(fname))
        };
        match file {
            Ok(f) => writers.push(Box::new(f)),
            Err(e) => { eprintln!("tee: {}: {}", fname, e); std::process::exit(1); }
        }
    }

    let stdin = io::stdin();
    let mut buf = [0u8; 8192];
    loop {
        let n = match stdio_read(&stdin, &mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        if io::stdout().write_all(&buf[..n]).is_err() { break; }
        for w in &mut writers {
            if w.write_all(&buf[..n]).is_err() { break; }
        }
    }
}

fn stdio_read(stdin: &io::Stdin, buf: &mut [u8]) -> io::Result<usize> {
    use std::io::Read;
    stdin.lock().read(buf)
}
