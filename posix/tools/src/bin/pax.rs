use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut mode: Option<char> = None;
    let mut archive_file: Option<String> = None;
    let mut files: Vec<&str> = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-w" => mode = Some('w'),
            "-r" => mode = Some('r'),
            "-x" => mode = Some('x'),
            "-f" if i + 1 < args.len() => { archive_file = Some(args[i+1].clone()); i += 1; }
            _ => {
                if !args[i].starts_with('-') {
                    files.push(&args[i]);
                }
            }
        }
        i += 1;
    }

    match mode {
        Some('w') => write_archive(archive_file.as_deref(), &files),
        Some('r') | Some('x') => read_archive(archive_file.as_deref(), mode == Some('x')),
        Some(_) => {}
        None => { eprintln!("pax: must specify -w, -r, or -x"); std::process::exit(1); }
    }
}

fn write_archive(archive: Option<&str>, files: &[&str]) {
    let output: Box<dyn Write> = match archive {
        Some(f) => Box::new(fs::File::create(f).unwrap_or_else(|e| {
            eprintln!("pax: cannot create {}: {}", f, e);
            std::process::exit(1);
        })),
        None => Box::new(io::stdout()),
    };
    let mut writer = output;
    for f in files {
        let path = Path::new(f);
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let full = format!("{}/{}", f, name);
                    write_file(&full, &mut writer);
                }
            }
        } else {
            write_file(f, &mut writer);
        }
    }
}

fn write_file(path: &str, writer: &mut dyn Write) {
    let data = fs::read(path).unwrap_or_default();
    let name = path.as_bytes();
    let mut header = [0u8; 512];
    let name_bytes = if name.len() > 99 { &name[..99] } else { name };
    header[..name_bytes.len()].copy_from_slice(name_bytes);
    let size_str = format!("{:011o}", data.len());
    header[124..135].copy_from_slice(size_str.as_bytes());
    let mtime_str = format!("{:011o}", 0u64);
    header[136..147].copy_from_slice(mtime_str.as_bytes());
    let mode_str = b"0000644\0";
    header[100..108].copy_from_slice(mode_str);

    let sum: u32 = header.iter().map(|&b| b as u32).sum();
    let csum_str = format!("{:06o}\0 ", sum);
    header[148..156].copy_from_slice(csum_str.as_bytes());

    writer.write_all(&header).ok();
    writer.write_all(&data).ok();
    let pad = (512 - data.len() % 512) % 512;
    if pad > 0 {
        writer.write_all(&vec![0u8; pad]).ok();
    }
}

fn read_archive(archive: Option<&str>, extract: bool) {
    let input: Box<dyn Read> = match archive {
        Some(f) => Box::new(fs::File::open(f).unwrap_or_else(|e| {
            eprintln!("pax: cannot open {}: {}", f, e);
            std::process::exit(1);
        })),
        None => Box::new(io::stdin()),
    };
    let mut reader = input;
    loop {
        let mut header = [0u8; 512];
        if reader.read_exact(&mut header).is_err() { break; }
        if header.iter().all(|&b| b == 0) { break; }

        let name_end = header.iter().position(|&b| b == 0).unwrap_or(99);
        let name = String::from_utf8_lossy(&header[..name_end]).to_string();
        if name.is_empty() { break; }

        let size_str = String::from_utf8_lossy(&header[124..135]).to_string();
        let size = usize::from_str_radix(size_str.trim(), 8).unwrap_or(0);

        if extract {
            let parent = Path::new(&name).parent().unwrap_or(Path::new("."));
            fs::create_dir_all(parent).ok();
            let mut data = vec![0u8; size];
            reader.read_exact(&mut data).ok();
            fs::write(&name, &data).ok();
        } else {
            println!("{}", name);
            let mut skip = size;
            let mut buf = [0u8; 4096];
            while skip > 0 {
                let n = reader.read(&mut buf[..skip.min(4096)]).unwrap_or(0);
                if n == 0 { break; }
                skip -= n;
            }
        }
        let pad = (512 - size % 512) % 512;
        if pad > 0 {
            let mut buf = vec![0u8; pad];
            reader.read_exact(&mut buf).ok();
        }
    }
}
