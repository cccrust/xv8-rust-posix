use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: iconv -f <from> -t <to> [file]");
        std::process::exit(1);
    }
    let mut from = String::new();
    let mut to = String::new();
    let mut file: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" if i + 1 < args.len() => { from = args[i+1].clone(); i += 2; }
            "-t" if i + 1 < args.len() => { to = args[i+1].clone(); i += 2; }
            "-l" => { list_codesets(); return; }
            _ => { file = Some(&args[i]); i += 1; }
        }
    }

    let data = if let Some(f) = file {
        let mut buf = Vec::new();
        std::fs::File::open(f).unwrap_or_else(|e| {
            eprintln!("iconv: {}: {}", f, e);
            std::process::exit(1);
        }).read_to_end(&mut buf).ok();
        buf
    } else {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf).ok();
        buf
    };

    let result = convert(&data, &from, &to);
    io::stdout().write_all(&result).ok();
}

fn convert(data: &[u8], from: &str, to: &str) -> Vec<u8> {
    let s = match from.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => String::from_utf8_lossy(data).to_string(),
        "ASCII" | "US-ASCII" => String::from_utf8_lossy(data).to_string(),
        "LATIN1" | "ISO-8859-1" => {
            data.iter().map(|&b| b as char).collect()
        }
        _ => String::from_utf8_lossy(data).to_string(),
    };
    match to.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => s.into_bytes(),
        "ASCII" | "US-ASCII" => {
            s.chars().map(|c| if c.is_ascii() { c as u8 } else { b'?' }).collect()
        }
        "LATIN1" | "ISO-8859-1" => {
            s.chars().map(|c| if c as u32 <= 255 { c as u8 } else { b'?' }).collect()
        }
        _ => s.into_bytes(),
    }
}

fn list_codesets() {
    println!("UTF-8");
    println!("ASCII");
    println!("LATIN1 (ISO-8859-1)");
}
