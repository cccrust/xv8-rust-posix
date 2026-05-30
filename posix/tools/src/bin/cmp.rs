use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut silent = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                's' => silent = true,
                _ => { eprintln!("cmp: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i + 2 > args.len() {
        eprintln!("usage: cmp [-s] file1 file2");
        std::process::exit(1);
    }

    let path1 = Path::new(&args[i]);
    let path2 = Path::new(&args[i + 1]);

    let mut f1 = File::open(path1).unwrap_or_else(|e| {
        eprintln!("cmp: {}: {}", path1.display(), e);
        std::process::exit(1);
    });
    let mut f2 = File::open(path2).unwrap_or_else(|e| {
        eprintln!("cmp: {}: {}", path2.display(), e);
        std::process::exit(1);
    });

    let mut buf1 = [0u8; 8192];
    let mut buf2 = [0u8; 8192];
    let mut offset: u64 = 1; // POSIX: 1-indexed

    loop {
        let n1 = f1.read(&mut buf1).unwrap_or(0);
        let n2 = f2.read(&mut buf2).unwrap_or(0);

        if n1 == 0 && n2 == 0 {
            return; // Files identical
        }

        if n1 != n2 || buf1[..n1.min(n2)] != buf2[..n1.min(n2)] {
            for j in 0..n1.min(n2) {
                if buf1[j] != buf2[j] {
                    if !silent {
                        println!("{} {} {:o} {:o}", path1.display(), offset + j as u64, buf1[j], buf2[j]);
                    }
                    std::process::exit(1);
                }
            }
            if !silent {
                println!("cmp: EOF on {}", if n1 < n2 { path1.display() } else { path2.display() });
            }
            std::process::exit(1);
        }

        offset += n1 as u64;
    }
}
