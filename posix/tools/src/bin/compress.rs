use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: compress <file>...");
        std::process::exit(1);
    }
    for file in &args[1..] {
        if file == "-d" || file == "-dc" || file == "-cd" {
            decompress_stdin();
            continue;
        }
        let data = std::fs::read(file).unwrap_or_else(|e| {
            eprintln!("compress: {}: {}", file, e);
            std::process::exit(1);
        });
        let compressed = lzw_compress(&data);
        let out_name = format!("{}.Z", file);
        std::fs::write(&out_name, &compressed).unwrap_or_else(|e| {
            eprintln!("compress: {}: {}", out_name, e);
            std::process::exit(1);
        });
    }
}

fn decompress_stdin() {
    let mut data = Vec::new();
    io::stdin().lock().read_to_end(&mut data).ok();
    let decompressed = lzw_decompress(&data);
    io::stdout().write_all(&decompressed).ok();
}

fn lzw_compress(data: &[u8]) -> Vec<u8> {
    use std::collections::HashMap;
    let mut dict: HashMap<Vec<u8>, u16> = HashMap::new();
    for i in 0..256u16 {
        dict.insert(vec![i as u8], i);
    }
    let mut next = 257u16;
    let mut out = Vec::new();
    let mut w = Vec::new();
    for &b in data {
        let mut wc = w.clone();
        wc.push(b);
        if dict.contains_key(&wc) {
            w = wc;
        } else {
            write_code(&mut out, dict[&w], next);
            if next < 4096 {
                dict.insert(wc, next);
                next += 1;
            }
            w = vec![b];
        }
    }
    if !w.is_empty() {
        write_code(&mut out, dict[&w], next);
    }
    out
}

fn write_code(out: &mut Vec<u8>, code: u16, _next: u16) {
    out.push((code >> 8) as u8);
    out.push(code as u8);
}

fn lzw_decompress(data: &[u8]) -> Vec<u8> {
    use std::collections::HashMap;
    let mut dict: HashMap<u16, Vec<u8>> = HashMap::new();
    for i in 0..256u16 {
        dict.insert(i, vec![i as u8]);
    }
    let mut next = 257u16;
    let mut out = Vec::new();
    if data.len() < 2 { return out; }
    let mut prev_code = ((data[0] as u16) << 8) | data[1] as u16;
    if let Some(entry) = dict.get(&prev_code) {
        out.extend_from_slice(entry);
    }
    let mut i = 2;
    while i + 1 < data.len() {
        let code = ((data[i] as u16) << 8) | data[i + 1] as u16;
        i += 2;
        let entry = if let Some(e) = dict.get(&code) {
            e.clone()
        } else if code == next {
            let mut e = dict.get(&prev_code).cloned().unwrap_or_default();
            e.push(e[0]);
            e
        } else {
            Vec::new()
        };
        out.extend_from_slice(&entry);
        if next < 4096 && !entry.is_empty() {
            let mut new_entry = dict.get(&prev_code).cloned().unwrap_or_default();
            new_entry.push(entry[0]);
            dict.insert(next, new_entry);
            next += 1;
        }
        prev_code = code;
    }
    out
}
