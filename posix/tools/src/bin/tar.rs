fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut create = false;
    let mut extract = false;
    let mut file = String::from("a.tar");
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'c' => create = true,
                'x' => extract = true,
                'f' => { i += 1; if i < args.len() { file = args[i].clone(); } }
                _ => { eprintln!("tar: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let files: Vec<&str> = args[i..].iter().map(String::as_str).collect();

    if create {
        // Create a simple tar file (uncompressed, just concatenation)
        let mut out = std::fs::File::create(&file).unwrap_or_else(|e| {
            eprintln!("tar: cannot create '{}': {}", file, e);
            std::process::exit(1);
        });
        for fname in &files {
            let path = std::path::Path::new(fname);
            if !path.exists() {
                eprintln!("tar: {}: does not exist", fname);
                continue;
            }
            let _meta = std::fs::metadata(path).unwrap();
            let data = std::fs::read(path).unwrap_or_default();

            // Write POSIX tar header (simplified: just name + size)
            let mut header = [0u8; 512];
            let name_bytes = fname.as_bytes();
            let len = name_bytes.len().min(100);
            header[..len].copy_from_slice(&name_bytes[..len]);

            // Write size in octal
            let size_str = format!("{:011o}", data.len());
            let size_bytes = size_str.as_bytes();
            let slen = size_bytes.len().min(12);
            header[124..124+slen].copy_from_slice(&size_bytes[..slen]);

            use std::io::Write;
            out.write_all(&header).unwrap();
            out.write_all(&data).unwrap();
            // Pad to 512
            let rem = (512 - (data.len() % 512)) % 512;
            if rem > 0 {
                out.write_all(&vec![0u8; rem]).unwrap();
            }
        }
        // Two zero blocks at end
        use std::io::Write;
        out.write_all(&[0u8; 1024]).unwrap();
    } else if extract {
        let data = std::fs::read(&file).unwrap_or_else(|e| {
            eprintln!("tar: cannot open '{}': {}", file, e);
            std::process::exit(1);
        });
        let mut pos = 0;
        while pos + 512 <= data.len() {
            let header = &data[pos..pos+512];
            if header.iter().all(|&b| b == 0) { break; }
            // Parse name
            let name_end = header.iter().position(|&b| b == 0).unwrap_or(100).min(100);
            let name = std::str::from_utf8(&header[..name_end]).unwrap_or("");
            if name.is_empty() { break; }

            // Parse size (octal at offset 124, 12 bytes)
            let size_str = std::str::from_utf8(&header[124..136]).unwrap_or("0").trim();
            let size = usize::from_str_radix(size_str, 8).unwrap_or(0);

            pos += 512;
            if pos + size > data.len() { break; }
            let file_data = &data[pos..pos+size];

            std::fs::write(name, file_data).unwrap_or_else(|e| {
                eprintln!("tar: cannot extract '{}': {}", name, e);
            });

            pos += ((size + 511) / 512) * 512;
        }
    } else {
        // List
        let data = std::fs::read(&file).unwrap_or_else(|e| {
            eprintln!("tar: cannot open '{}': {}", file, e);
            std::process::exit(1);
        });
        let mut pos = 0;
        while pos + 512 <= data.len() {
            let header = &data[pos..pos+512];
            if header.iter().all(|&b| b == 0) { break; }
            let name_end = header.iter().position(|&b| b == 0).unwrap_or(100).min(100);
            let name = std::str::from_utf8(&header[..name_end]).unwrap_or("");
            if name.is_empty() { break; }
            println!("{}", name);
            let size_str = std::str::from_utf8(&header[124..136]).unwrap_or("0").trim();
            let size = usize::from_str_radix(size_str, 8).unwrap_or(0);
            pos += 512 + ((size + 511) / 512) * 512;
        }
    }
}
