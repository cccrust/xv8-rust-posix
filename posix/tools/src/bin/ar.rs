use std::fs;

const ARMAG: &[u8; 8] = b"!<arch>\n";
const ARFMAG: &[u8; 2] = b"`\n";

struct ArEntry {
    name: String,
    data: Vec<u8>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ar [-t|-x|-r|-c|-v|-s|-u] archive [files...]");
        std::process::exit(1);
    }
    let mut i = 1;
    let mut op = 't';
    let mut verbose = false;
    let mut create = false;
    while i < args.len() && args[i].starts_with('-') && args[i].len() > 1 {
        let flags = &args[i][1..];
        for ch in flags.chars() {
            match ch {
                't' | 'x' | 'r' | 'd' | 'c' | 'v' | 's' | 'u' => {
                    if ch != 'c' && ch != 'v' && ch != 's' && ch != 'u' {
                        op = ch;
                    }
                    if ch == 'c' { create = true; }
                    if ch == 'v' { verbose = true; }
                }
                _ => {}
            }
        }
        i += 1;
    }
    if i >= args.len() {
        eprintln!("ar: missing archive name");
        std::process::exit(1);
    }
    let archive = &args[i];
    i += 1;
    let files: Vec<&str> = args[i..].iter().map(|s| s.as_str()).collect();

    match op {
        't' => list_archive(archive, files, verbose),
        'x' => extract_archive(archive, files, verbose),
        'r' => replace_archive(archive, files, create, verbose),
        _ => {
            eprintln!("ar: unsupported operation {}", op);
            std::process::exit(1);
        }
    }
}

fn read_archive(path: &str) -> Vec<ArEntry> {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    if !data.starts_with(ARMAG) {
        return Vec::new();
    }
    let mut entries = Vec::new();
    let mut pos = 8;
    while pos + 60 <= data.len() {
        let header = &data[pos..pos+60];
        let name_bytes = &header[0..16];
        let size_bytes = &header[48..58];
        let fmag = &header[58..60];
        if fmag != ARFMAG {
            break;
        }
        let mut name = String::from_utf8_lossy(name_bytes).trim().to_string();
        let size: usize = String::from_utf8_lossy(size_bytes).trim().parse().unwrap_or(0);
        pos += 60;
        if pos + size > data.len() {
            break;
        }
        // BSD long name: #1/<len>
        if name.starts_with("#1/") {
            let len: usize = name[3..].parse().unwrap_or(0);
            if len <= size {
                name = String::from_utf8_lossy(&data[pos..pos+len]).trim_matches('\0').to_string();
            }
            let entry_data = data[pos+len..pos+size].to_vec();
            entries.push(ArEntry { name, data: entry_data });
            pos += size;
        } else {
            let entry_data = data[pos..pos+size].to_vec();
            entries.push(ArEntry { name, data: entry_data });
            pos += size;
        }
        if pos % 2 == 1 { pos += 1; } // padding to even
    }
    entries
}

fn write_archive(path: &str, entries: &[ArEntry]) {
    let mut out = Vec::new();
    out.extend_from_slice(ARMAG);
    for e in entries {
        let is_long = e.name.len() > 15;
        let name = if is_long {
            format!("#1/{}", e.name.len())
        } else {
            e.name.clone()
        };
        let mut hdr = vec![b' '; 60];
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(16);
        hdr[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        let date = format!("{:12}", 0);
        let uid = format!("{:6}", 0);
        let gid = format!("{:6}", 0);
        let mode = format!("{:8o}", 0o100644u32);
        let total_size = if is_long { e.name.len() + e.data.len() } else { e.data.len() };
        let size = format!("{:10}", total_size);
        hdr[16..28].copy_from_slice(date.as_bytes());
        hdr[28..34].copy_from_slice(uid.as_bytes());
        hdr[34..40].copy_from_slice(gid.as_bytes());
        hdr[40..48].copy_from_slice(mode.as_bytes());
        hdr[48..58].copy_from_slice(size.as_bytes());
        hdr[58..60].copy_from_slice(ARFMAG);
        out.extend_from_slice(&hdr);
        if is_long {
            let name_null = format!("{:\0<1$}", e.name, e.name.len());
            out.extend_from_slice(name_null.as_bytes());
        }
        out.extend_from_slice(&e.data);
        if out.len() % 2 == 1 {
            out.push(b'\n');
        }
    }
    if let Err(e) = fs::write(path, &out) {
        eprintln!("ar: cannot write {}: {}", path, e);
        std::process::exit(1);
    }
}

fn list_archive(archive: &str, files: Vec<&str>, verbose: bool) {
    let entries = read_archive(archive);
    if entries.is_empty() && std::path::Path::new(archive).exists() {
        eprintln!("ar: {}: invalid archive", archive);
        std::process::exit(1);
    }
    for e in &entries {
        if files.is_empty() || files.contains(&e.name.as_str()) {
            if verbose {
                println!("rw-r--r-- 0/0 {:10} {}", e.data.len(), e.name);
            } else {
                println!("{}", e.name);
            }
        }
    }
}

fn extract_archive(archive: &str, files: Vec<&str>, verbose: bool) {
    let entries = read_archive(archive);
    for e in &entries {
        if files.is_empty() || files.contains(&e.name.as_str()) {
            if verbose {
                eprintln!("x - {}", e.name);
            }
            let _ = fs::write(&e.name, &e.data);
        }
    }
}

fn replace_archive(archive: &str, files: Vec<&str>, create: bool, verbose: bool) {
    let mut entries = read_archive(archive);
    if !std::path::Path::new(archive).exists() && !create {
        eprintln!("ar: {}: No such file or directory", archive);
        std::process::exit(1);
    }
    for fname in &files {
        let data = match fs::read(fname) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("ar: {}: {}", fname, e);
                continue;
            }
        };
        if let Some(pos) = entries.iter().position(|e| e.name == *fname) {
            entries[pos].data = data;
            if verbose {
                eprintln!("r - {}", fname);
            }
        } else {
            entries.push(ArEntry { name: fname.to_string(), data });
            if verbose {
                eprintln!("a - {}", fname);
            }
        }
    }
    write_archive(archive, &entries);
}
