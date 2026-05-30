use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut symbolic = false;
    let mut force = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                's' => symbolic = true,
                'f' => force = true,
                _ => { eprintln!("ln: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let targets: Vec<String> = args[i..].to_vec();
    if targets.len() < 2 {
        eprintln!("usage: ln [-sf] target ... link_name");
        std::process::exit(1);
    }

    let link_name = Path::new(&targets[targets.len() - 1]);
    let sources = &targets[..targets.len() - 1];
    let link_is_dir = link_name.is_dir();

    for src_str in sources {
        let src = Path::new(src_str);
        let link = if link_is_dir {
            link_name.join(src.file_name().unwrap_or_default())
        } else {
            link_name.to_path_buf()
        };

        if link.exists() {
            if force {
                let _ = fs::remove_file(&link);
            } else {
                eprintln!("ln: '{}' exists (use -f to force)", link.display());
                std::process::exit(1);
            }
        }

        if symbolic {
            if let Err(e) = std::os::unix::fs::symlink(src, &link) {
                eprintln!("ln: cannot create symlink '{}' -> '{}': {}", link.display(), src.display(), e);
                std::process::exit(1);
            }
        } else {
            if let Err(e) = fs::hard_link(src, &link) {
                eprintln!("ln: cannot create link '{}' -> '{}': {}", link.display(), src.display(), e);
                std::process::exit(1);
            }
        }
    }
}
