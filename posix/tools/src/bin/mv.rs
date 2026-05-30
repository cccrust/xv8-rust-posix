use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut interactive = false;
    let mut force = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'i' => interactive = true,
                'f' => force = true,
                _ => { eprintln!("mv: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let targets: Vec<String> = args[i..].to_vec();
    if targets.len() < 2 {
        eprintln!("usage: mv [-if] source ... target");
        std::process::exit(1);
    }

    let target = Path::new(&targets[targets.len() - 1]);
    let sources = &targets[..targets.len() - 1];
    let target_is_dir = target.is_dir();

    for src_str in sources {
        let src = Path::new(src_str);
        let dst = if target_is_dir {
            target.join(src.file_name().unwrap_or_default())
        } else {
            target.to_path_buf()
        };

        if dst.exists() && !force {
            if interactive {
                eprint!("mv: overwrite '{}'? ", dst.display());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if !input.trim().eq_ignore_ascii_case("y") { continue; }
            } else {
                eprintln!("mv: '{}' exists (use -f to force)", dst.display());
                std::process::exit(1);
            }
        }

        if let Err(e) = fs::rename(src, &dst) {
            // Cross-device move: copy + remove
            if src.is_dir() {
                eprintln!("mv: cannot move '{}': {}", src.display(), e);
                std::process::exit(1);
            }
            if let Err(e2) = fs::copy(src, &dst) {
                eprintln!("mv: cannot copy '{}': {}", src.display(), e2);
                std::process::exit(1);
            }
            if let Err(e2) = fs::remove_file(src) {
                eprintln!("mv: cannot remove '{}': {}", src.display(), e2);
                std::process::exit(1);
            }
        }
    }
}
