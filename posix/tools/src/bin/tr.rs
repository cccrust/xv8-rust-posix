use std::io::{self, Read, Write};

fn expand_set(s: &str) -> Vec<u8> {
    let mut set = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
            for c in bytes[i]..=bytes[i + 2] {
                set.push(c);
            }
            i += 3;
        } else {
            set.push(bytes[i]);
            i += 1;
        }
    }
    set
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut delete = false;
    let mut squeeze = false;
    let mut complement = false;
    let mut set1 = String::new();
    let mut set2 = String::new();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'd' => delete = true,
                's' => squeeze = true,
                'c' => complement = true,
                _ => { eprintln!("tr: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i < args.len() { set1 = args[i].clone(); i += 1; }
    if i < args.len() { set2 = args[i].clone(); }

    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap_or(0);

    let set1_bytes = expand_set(&set1);
    let set2_bytes = expand_set(&set2);

    // Determine which bytes are "active" for delete or translation
    let active = if complement {
        let mut all: Vec<u8> = (0..=255).collect();
        all.retain(|b| !set1_bytes.contains(b));
        all
    } else {
        set1_bytes.clone()
    };

    // First pass: delete or translate
    let mut mid = Vec::new();
    if delete {
        let mut map = [true; 256];
        for &b in &active { map[b as usize] = false; }
        for &b in &input {
            if map[b as usize] { mid.push(b); }
        }
    } else if !set2_bytes.is_empty() {
        let mut map = [0u8; 256];
        for i in 0..=255u8 { map[i as usize] = i; }
        for (i, &b) in active.iter().enumerate() {
            map[b as usize] = if i < set2_bytes.len() { set2_bytes[i] } else { *set2_bytes.last().unwrap_or(&b) };
        }
        for &b in &input {
            if delete || !active.contains(&b) {
                mid.push(b);
            } else {
                mid.push(map[b as usize]);
            }
        }
    } else {
        mid = input;
    }

    // Second pass: squeeze
    if squeeze {
        let mut out = Vec::new();
        let mut prev: Option<u8> = None;
        for &b in &mid {
            let in_set = active.contains(&b);
            if in_set && prev == Some(b) { continue; }
            out.push(b);
            prev = Some(b);
        }
        io::stdout().write_all(&out).unwrap();
    } else {
        io::stdout().write_all(&mid).unwrap();
    }
}
