use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut human = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'h' => human = true,
                _ => { eprintln!("df: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    #[cfg(unix)]
    {
        // List mount points from /proc/mounts or mount output
        let mounts = Path::new("/proc/mounts");
        let _content = if mounts.exists() {
            std::fs::read_to_string(mounts).unwrap_or_default()
        } else {
            String::new()
        };

        // Fallback: just stat known filesystems
        let paths: Vec<&str> = if i < args.len() { vec![&args[i]] } else { vec!["/"] };

        for p in &paths {
            let path = Path::new(p);
            if let Ok(_stat) = std::fs::metadata(path) {
                if human {
                    println!("Filesystem     1M-blocks  Used Available Use% Mounted on");
                    // Statfs would be more accurate; use metadata as approximation
                    println!("{:<15}        -     -         -    - {}", "?", p);
                } else {
                    println!("Filesystem     1024-blocks  Used Available Use% Mounted on");
                    println!("{:<15}        -     -         -    - {}", "?", p);
                }
            } else {
                eprintln!("df: {}: cannot access", p);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(unix))]
    {
        eprintln!("df: not supported on this platform");
        std::process::exit(1);
    }
}
