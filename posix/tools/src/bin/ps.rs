use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut all = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { break; }
        for c in args[i][1..].chars() {
            match c {
                'a' => all = true,
                _ => { eprintln!("ps: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if all {
        #[cfg(unix)]
        {
            let proc_dir = Path::new("/proc");
            if proc_dir.is_dir() {
                for entry in std::fs::read_dir(proc_dir).unwrap() {
                    if let Ok(entry) = entry {
                        let name = entry.file_name();
                        if let Ok(pid) = name.to_string_lossy().parse::<u32>() {
                            let stat_path = proc_dir.join(&name).join("stat");
                            if let Ok(content) = std::fs::read_to_string(&stat_path) {
                                let fields: Vec<&str> = content.splitn(4, ' ').collect();
                                if fields.len() >= 2 {
                                    let comm = fields[1].trim_matches('(').trim_matches(')');
                                    println!("{:>5} {}", pid, comm);
                                }
                            }
                        }
                    }
                }
            } else {
                eprintln!("ps: /proc not available");
                std::process::exit(1);
            }
        }
        #[cfg(not(unix))]
        {
            eprintln!("ps: not supported on this platform");
            std::process::exit(1);
        }
    } else {
        // Default: show current process only
        let pid = std::process::id();
        let name = std::env::args().next().unwrap_or_else(|| "?".to_string());
        println!("{:>5} {}", pid, name);
    }
}
