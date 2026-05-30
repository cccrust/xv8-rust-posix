use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut parents = false;
    let mut mode: Option<u32> = None;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'p' => parents = true,
                'm' => {
                    i += 1;
                    if i < args.len() {
                        mode = Some(u32::from_str_radix(&args[i], 8).unwrap_or(0o777));
                    }
                }
                _ => { eprintln!("mkdir: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: mkdir [-p] [-m mode] directory ...");
        std::process::exit(1);
    }

    for path_str in &args[i..] {
        let path = Path::new(path_str);
        if parents {
            if let Err(e) = fs::create_dir_all(path) {
                eprintln!("mkdir: cannot create directory '{}': {}", path.display(), e);
                std::process::exit(1);
            }
        } else {
            if let Err(e) = fs::create_dir(path) {
                eprintln!("mkdir: cannot create directory '{}': {}", path.display(), e);
                std::process::exit(1);
            }
        }
        if let Some(m) = mode {
            let _ = fs::set_permissions(path, std::fs::Permissions::from_mode(m & 0o777));
        }
    }
}
