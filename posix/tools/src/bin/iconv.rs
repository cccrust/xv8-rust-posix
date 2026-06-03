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

fn canonicalize(enc: &str) -> &str {
    match enc.to_uppercase().as_str() {
        "UTF8" | "UTF-8" => "UTF-8",
        "UTF16" | "UTF-16" | "UTF-16LE" | "UTF16LE" => "UTF-16LE",
        "UTF-16BE" | "UTF16BE" => "UTF-16BE",
        "UTF32" | "UTF-32" | "UTF-32LE" | "UTF32LE" => "UTF-32LE",
        "UTF-32BE" | "UTF32BE" => "UTF-32BE",
        "ASCII" | "US-ASCII" => "ASCII",
        "LATIN1" | "ISO-8859-1" | "ISO8859-1" | "ISO_8859-1" => "ISO-8859-1",
        "LATIN2" | "ISO-8859-2" | "ISO8859-2" | "ISO_8859-2" => "ISO-8859-2",
        "LATIN3" | "ISO-8859-3" | "ISO8859-3" | "ISO_8859-3" => "ISO-8859-3",
        "LATIN4" | "ISO-8859-4" | "ISO8859-4" | "ISO_8859-4" => "ISO-8859-4",
        "LATIN5" | "ISO-8859-9" | "ISO8859-9" | "ISO_8859-9" => "ISO-8859-9",
        "LATIN9" | "ISO-8859-15" | "ISO8859-15" | "ISO_8859-15" => "ISO-8859-15",
        "CP1252" | "WINDOWS-1252" => "CP1252",
        "CP437" | "IBM437" => "CP437",
        "CP850" | "IBM850" => "CP850",
        "KOI8-R" | "KOI8R" => "KOI8-R",
        _ => enc,
    }
}

fn convert(data: &[u8], from: &str, to: &str) -> Vec<u8> {
    let from = canonicalize(from);
    let to = canonicalize(to);
    if from == to {
        return data.to_vec();
    }
    let codepoints = decode(data, from);
    encode(&codepoints, to)
}

fn decode(data: &[u8], enc: &str) -> Vec<u32> {
    match enc {
        "UTF-8" => {
            let s = String::from_utf8_lossy(data);
            s.chars().map(|c| c as u32).collect()
        }
        "UTF-16LE" => {
            let mut cp = Vec::new();
            let mut i = 0;
            while i + 1 < data.len() {
                let c = data[i] as u32 | ((data[i + 1] as u32) << 8);
                i += 2;
                if c >= 0xD800 && c <= 0xDBFF && i + 1 < data.len() {
                    let lo = data[i] as u32 | ((data[i + 1] as u32) << 8);
                    i += 2;
                    cp.push(0x10000 + ((c - 0xD800) << 10) + (lo - 0xDC00));
                } else {
                    cp.push(c);
                }
            }
            cp
        }
        "UTF-16BE" => {
            let mut cp = Vec::new();
            let mut i = 0;
            while i + 1 < data.len() {
                let c = (data[i] as u32) << 8 | data[i + 1] as u32;
                i += 2;
                if c >= 0xD800 && c <= 0xDBFF && i + 1 < data.len() {
                    let lo = (data[i] as u32) << 8 | data[i + 1] as u32;
                    i += 2;
                    cp.push(0x10000 + ((c - 0xD800) << 10) + (lo - 0xDC00));
                } else {
                    cp.push(c);
                }
            }
            cp
        }
        "UTF-32LE" => {
            data.chunks(4).map(|ch| {
                ch.iter().enumerate().fold(0u32, |acc, (j, &b)| acc | (b as u32) << (j * 8))
            }).collect()
        }
        "UTF-32BE" => {
            data.chunks(4).map(|ch| {
                ch.iter().enumerate().fold(0u32, |acc, (j, &b)| acc | (b as u32) << ((3 - j) * 8))
            }).collect()
        }
        "ASCII" => data.iter().map(|&b| b as u32).collect(),
        "ISO-8859-1" => data.iter().map(|&b| b as u32).collect(),
        enc => data.iter().map(|&b| b as u32).collect(),
    }
}

fn encode(codepoints: &[u32], enc: &str) -> Vec<u8> {
    match enc {
        "UTF-8" => {
            codepoints.iter().flat_map(|&cp| {
                if cp <= 0x7F { vec![cp as u8] }
                else if cp <= 0x7FF { vec![0xC0 | (cp >> 6) as u8, 0x80 | (cp & 0x3F) as u8] }
                else if cp <= 0xFFFF { vec![0xE0 | (cp >> 12) as u8, 0x80 | ((cp >> 6) & 0x3F) as u8, 0x80 | (cp & 0x3F) as u8] }
                else { vec![0xF0 | (cp >> 18) as u8, 0x80 | ((cp >> 12) & 0x3F) as u8, 0x80 | ((cp >> 6) & 0x3F) as u8, 0x80 | (cp & 0x3F) as u8] }
            }).collect()
        }
        "UTF-16LE" => {
            let mut out = Vec::new();
            for &cp in codepoints {
                if cp <= 0xFFFF {
                    out.extend_from_slice(&[cp as u8, (cp >> 8) as u8]);
                } else {
                    let cp = cp - 0x10000;
                    let hi = 0xD800 | (cp >> 10);
                    let lo = 0xDC00 | (cp & 0x3FF);
                    out.extend_from_slice(&[hi as u8, (hi >> 8) as u8, lo as u8, (lo >> 8) as u8]);
                }
            }
            out
        }
        "UTF-16BE" => {
            let mut out = Vec::new();
            for &cp in codepoints {
                if cp <= 0xFFFF {
                    out.extend_from_slice(&[(cp >> 8) as u8, cp as u8]);
                } else {
                    let cp = cp - 0x10000;
                    let hi = 0xD800 | (cp >> 10);
                    let lo = 0xDC00 | (cp & 0x3FF);
                    out.extend_from_slice(&[(hi >> 8) as u8, hi as u8, (lo >> 8) as u8, lo as u8]);
                }
            }
            out
        }
        "UTF-32LE" => {
            codepoints.iter().flat_map(|&cp| {
                vec![cp as u8, (cp >> 8) as u8, (cp >> 16) as u8, (cp >> 24) as u8]
            }).collect()
        }
        "UTF-32BE" => {
            codepoints.iter().flat_map(|&cp| {
                vec![(cp >> 24) as u8, (cp >> 16) as u8, (cp >> 8) as u8, cp as u8]
            }).collect()
        }
        "ASCII" => {
            codepoints.iter().map(|&cp| if cp <= 0x7F { cp as u8 } else { b'?' }).collect()
        }
        enc => {
            codepoints.iter().map(|&cp| {
                if cp <= 0xFF { cp as u8 } else { b'?' }
            }).collect()
        }
    }
}

fn list_codesets() {
    let sets = [
        "UTF-8", "UTF-16LE", "UTF-16BE", "UTF-32LE", "UTF-32BE",
        "ASCII",
        "ISO-8859-1 (Latin-1 Western European)",
        "ISO-8859-2 (Latin-2 Central European)",
        "ISO-8859-3 (Latin-3 South European)",
        "ISO-8859-4 (Latin-4 North European)",
        "ISO-8859-9 (Latin-5 Turkish)",
        "ISO-8859-15 (Latin-9 Western European with €)",
        "CP1252 (Windows Western)",
        "CP437 (DOS Original)",
        "CP850 (DOS Latin-1)",
        "KOI8-R (Russian)",
    ];
    for s in &sets {
        println!("{}", s);
    }
}
