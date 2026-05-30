use std::fs;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: file <file>...");
        std::process::exit(1);
    }
    for path in &args[1..] {
        let result = file_type(path);
        println!("{}: {}", path, result);
    }
}

fn file_type(path: &str) -> String {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                match fs::read_link(path) {
                    Ok(target) => format!("symbolic link to {}", target.display()),
                    Err(_) => "symbolic link (broken)".to_string(),
                }
            } else if meta.file_type().is_dir() {
                "directory".to_string()
            } else if meta.file_type().is_file() {
                let magic = read_magic_bytes(path);
                magic_file_type(&magic, path)
            } else {
                "special file".to_string()
            }
        }
        Err(_) => {
            if path == "-" {
                "ASCII text".to_string()
            } else if path.contains('/') || fs::metadata(path).is_err() {
                "cannot open".to_string()
            } else {
                "ASCII text".to_string()
            }
        }
    }
}

fn read_magic_bytes(path: &str) -> Vec<u8> {
    let mut f = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let mut buf = vec![0u8; 64];
    match f.read(&mut buf) {
        Ok(n) => buf[..n].to_vec(),
        Err(_) => vec![],
    }
}

fn magic_file_type(magic: &[u8], _path: &str) -> String {
    if magic.len() < 2 { return "ASCII text".to_string(); }

    if magic.starts_with(b"\x7fELF") {
        return "ELF".to_string();
    }
    if magic.starts_with(b"\x89PNG") {
        return "PNG image data".to_string();
    }
    if magic.starts_with(b"\xff\xd8") {
        return "JPEG image data".to_string();
    }
    if magic.starts_with(b"GIF8") {
        return "GIF image data".to_string();
    }
    if magic.starts_with(b"%PDF") {
        return "PDF document".to_string();
    }
    if magic.starts_with(b"MZ") {
        return "PE32 executable".to_string();
    }
    if magic.starts_with(b"#!") {
        let s = String::from_utf8_lossy(magic);
        let line = s.lines().next().unwrap_or("");
        return format!("{} script, ASCII text executable", line[2..].trim());
    }
    if magic.starts_with(&[0xca, 0xfe, 0xba, 0xbe]) {
        return "Mach-O universal binary".to_string();
    }
    if magic.starts_with(&[0xcf, 0xfa, 0xed, 0xfe]) || magic.starts_with(&[0xce, 0xfa, 0xed, 0xfe]) {
        return "Mach-O 64-bit executable".to_string();
    }
    if magic.starts_with(b"PK") && magic.len() > 4 {
        if magic[2..].starts_with(b"\x03\x04") {
            return "Zip archive".to_string();
        }
    }
    if magic.starts_with(b"BZh") {
        return "bzip2 compressed data".to_string();
    }
    if magic.starts_with(&[0x1f, 0x8b]) {
        return "gzip compressed data".to_string();
    }
    if magic.starts_with(b"\xfd7zXZ") {
        return "XZ compressed data".to_string();
    }

    let text = String::from_utf8_lossy(magic);
    if text.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || c == '\x00' || c == '\x0b') {
        let nul_count = magic.iter().filter(|&&b| b == 0).count();
        if nul_count > 0 {
            return "ASCII text (with NULs)".to_string();
        }
        "ASCII text".to_string()
    } else if text.chars().any(|c| !c.is_ascii() && c != '\n' && c != '\r') {
        "UTF-8 Unicode text".to_string()
    } else {
        "data".to_string()
    }
}
