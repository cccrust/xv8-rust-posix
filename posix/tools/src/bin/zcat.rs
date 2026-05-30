use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        let mut data = Vec::new();
        io::stdin().lock().read_to_end(&mut data).ok();
        let decompressed = lzw_decompress(&data);
        io::stdout().write_all(&decompressed).ok();
        return;
    }
    for file in &args[1..] {
        let data = std::fs::read(file).unwrap_or_else(|e| {
            eprintln!("zcat: {}: {}", file, e);
            std::process::exit(1);
        });
        let decompressed = lzw_decompress(&data);
        io::stdout().write_all(&decompressed).ok();
    }
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
