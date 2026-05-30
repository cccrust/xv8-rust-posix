use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::Path;

fn tail_file(path: &Path, lines: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut all_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        all_lines.push(line?);
    }
    let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
    for line in &all_lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

fn tail_bytes(path: &Path, nbytes: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let len = file.seek(SeekFrom::End(0))?;
    let start = if (len as usize) > nbytes { len as usize - nbytes } else { 0 };
    let mut buf = vec![0u8; (len as usize - start).min(nbytes)];
    file.seek(SeekFrom::Start(start as u64))?;
    file.read_exact(&mut buf)?;
    io::stdout().write_all(&buf)?;
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
        let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
        for line in &all_lines[start..] {
            println!("{}", line);
        }
        return;
    }

    for (idx, fname) in files.iter().enumerate() {
        let path = Path::new(fname);
        if files.len() > 1 {
            println!("{}==> {} <==", if idx > 0 { "\n" } else { "" }, fname);
        }
        let result = if let Some(nbytes) = bytes {
            tail_bytes(path, nbytes)
        } else {
            tail_file(path, lines)
        };
        if let Err(e) = result {
            eprintln!("tail: {}: {}", fname, e);
            std::process::exit(1);
        }
    }
}
