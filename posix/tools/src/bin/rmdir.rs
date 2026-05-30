use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut parents = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'p' => parents = true,
                _ => { eprintln!("rmdir: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: rmdir [-p] directory ...");
        std::process::exit(1);
    }

    for path_str in &args[i..] {
        let path = Path::new(path_str);
        if let Err(e) = fs::remove_dir(path) {
            eprintln!("rmdir: failed to remove '{}': {}", path.display(), e);
            std::process::exit(1);
        }
        if parents {
            // Remove empty parent directories
            let mut p = path.parent();
            while let Some(parent) = p {
                if parent.as_os_str().is_empty() || parent.to_string_lossy() == "/" { break; }
                if fs::remove_dir(parent).is_err() { break; }
                p = parent.parent();
            }
        }
    }
}
