use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::Path;

fn tail_file_from_start(path: &Path, start_line: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i >= start_line {
            println!("{}", line);
        }
    }
    Ok(())
}

fn tail_file_from_end(path: &Path, nlines: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut all_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        all_lines.push(line?);
    }
    let start = if all_lines.len() > nlines { all_lines.len() - nlines } else { 0 };
    for line in &all_lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

fn tail_bytes_from_start(path: &Path, start_byte: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start_byte as u64))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    io::stdout().write_all(&buf)?;
    Ok(())
}

fn tail_bytes_from_end(path: &Path, nbytes: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let len = file.seek(SeekFrom::End(0))?;
    let start = if (len as usize) > nbytes { len as usize - nbytes } else { 0 };
    let n = (len as usize - start).min(nbytes);
    let mut buf = vec![0u8; n];
    file.seek(SeekFrom::Start(start as u64))?;
    file.read_exact(&mut buf)?;
    io::stdout().write_all(&buf)?;
    Ok(())
}

fn parse_count(s: &str, default: usize) -> (usize, bool) {
    if let Some(rest) = s.strip_prefix('+') {
        (rest.parse().unwrap_or(default), true)
    } else {
        (s.parse().unwrap_or(default), false)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut lines: usize = 10;
    let mut bytes: Option<(usize, bool)> = None;
    let mut from_start = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'n' => {
                    i += 1;
                    if i < args.len() {
                        let parsed = parse_count(&args[i], 10);
                        lines = parsed.0;
                        if parsed.1 { from_start = true; }
                    }
                }
                'c' => {
                    i += 1;
                    if i < args.len() {
                        let parsed = parse_count(&args[i], 0);
                        bytes = Some((parsed.0, parsed.1));
                        if parsed.1 { from_start = true; }
                    }
                }
                'q' => {}
                'v' => {}
                _ => { eprintln!("tail: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<String> = args[i..].to_vec();

    if files.is_empty() {
        let stdin = io::stdin();
        let mut all_lines: Vec<String> = Vec::new();
        for line in stdin.lock().lines() {
            all_lines.push(line.unwrap_or_default());
        }
        if from_start {
            for line in &all_lines[(lines - 1).min(all_lines.len())..] {
                println!("{}", line);
            }
        } else {
            let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
            for line in &all_lines[start..] {
                println!("{}", line);
            }
        }
        return;
    }

    for (idx, fname) in files.iter().enumerate() {
        let path = Path::new(fname);
        if files.len() > 1 {
            println!("{}==> {} <==", if idx > 0 { "\n" } else { "" }, fname);
        }
        let result = if let Some((n, from_start_bytes)) = bytes {
            if from_start_bytes {
                tail_bytes_from_start(path, n - 1)
            } else {
                tail_bytes_from_end(path, n)
            }
        } else if from_start {
            tail_file_from_start(path, lines - 1)
        } else {
            tail_file_from_end(path, lines)
        };
        if let Err(e) = result {
            eprintln!("tail: {}: {}", fname, e);
            std::process::exit(1);
        }
    }
}
