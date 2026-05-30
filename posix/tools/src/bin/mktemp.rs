fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut dir = false;
    let mut template = "tmp.XXXXXX";
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--directory" => { dir = true; }
            "-t" | "--tmpdir" => { i += 1; } // ignored, always /tmp
            "-p" | "--prefix" => { i += 1; } // ignored
            _ => { template = &args[i]; }
        }
        i += 1;
    }
    let prefix = if template.contains('X') {
        let idx = template.find('X').unwrap();
        &template[..idx]
    } else {
        template
    };
    // Generate unique temp name
    for _attempt in 0..1000 {
        let suffix: String = (0..6).map(|_| {
            let c = b"abcdefghijklmnopqrstuvwxyz0123456789"[rand() as usize % 36];
            c as char
        }).collect();
        let name = format!("/tmp/{}{}", prefix, suffix);
        if dir {
            match std::fs::create_dir(&name) {
                Ok(()) => { println!("{}", name); return; }
                Err(_) => continue,
            }
        } else {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&name) {
                Ok(_) => { println!("{}", name); return; }
                Err(_) => continue,
            }
        }
    }
    eprintln!("mktemp: cannot create temp file");
    std::process::exit(1);
}

// Simple LCG random
fn rand() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
    let mut seed = nanos.wrapping_mul(1103515245).wrapping_add(12345);
    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    seed
}
