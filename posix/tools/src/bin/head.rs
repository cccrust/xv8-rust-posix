use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn head_file(path: &Path, lines: usize, bytes: Option<usize>) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    if let Some(nbytes) = bytes {
        let mut buf = vec![0u8; nbytes];
        let mut handle = reader.into_inner();
        let n = std::io::Read::read(&mut handle, &mut buf)?;
        io::stdout().write_all(&buf[..n])?;
    } else {
        for (i, line) in reader.lines().enumerate() {
            if i >= lines { break; }
            println!("{}", line?);
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut lines: usize = 10;
    let mut bytes: Option<usize> = None;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'n' => { i += 1; if i < args.len() { lines = args[i].parse().unwrap_or(10); } }
                'c' => { i += 1; if i < args.len() { bytes = Some(args[i].parse().unwrap_or(0)); } }
                'q' => {}
                'v' => {}
                _ => { eprintln!("head: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<String> = args[i..].to_vec();

    if files.is_empty() {
        let reader = io::stdin();
        for (i, line) in reader.lock().lines().enumerate() {
            if i >= lines { break; }
            println!("{}", line.unwrap_or_default());
        }
        return;
    }

    for (idx, fname) in files.iter().enumerate() {
        let path = Path::new(fname);
        if files.len() > 1 {
            println!("{}==> {} <==", if idx > 0 { "\n" } else { "" }, fname);
        }
        if let Err(e) = head_file(path, lines, bytes) {
            eprintln!("head: {}: {}", fname, e);
            std::process::exit(1);
        }
    }
}
