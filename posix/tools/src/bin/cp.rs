use std::fs;
use std::path::Path;

fn copy_file(src: &Path, dst: &Path, preserve: bool) -> Result<(), String> {
    fs::copy(src, dst).map_err(|e| format!("{}", e))?;
    if preserve {
        if let Ok(meta) = fs::metadata(src) {
            let _ = fs::set_permissions(dst, meta.permissions());
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path, recursive: bool, preserve: bool, interactive: bool, force: bool) -> Result<(), String> {
    if !recursive {
        return Err(format!("omitting directory '{}'", src.display()));
    }
    fs::create_dir_all(dst).map_err(|e| format!("cannot create '{}': {}", dst.display(), e))?;
    let entries = fs::read_dir(src).map_err(|e| format!("cannot read '{}': {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            copy_dir(&src_path, &dst_path, true, preserve, interactive, force)?;
        } else {
            if dst_path.exists() && !force {
                if interactive {
                    eprint!("cp: overwrite '{}'? ", dst_path.display());
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if !input.trim().eq_ignore_ascii_case("y") { continue; }
                } else {
                    return Err(format!("'{}' exists", dst_path.display()));
                }
            }
            copy_file(&src_path, &dst_path, preserve)?;
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut recursive = false;
    let mut preserve = false;
    let mut interactive = false;
    let mut force = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'R' | 'r' => recursive = true,
                'p' => preserve = true,
                'i' => interactive = true,
                'f' => force = true,
                _ => { eprintln!("cp: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let srcs: Vec<String> = args[i..].to_vec();
    if srcs.len() < 2 {
        eprintln!("usage: cp [-Ripf] source ... target");
        std::process::exit(1);
    }

    let target = Path::new(&srcs[srcs.len() - 1]);
    let sources = &srcs[..srcs.len() - 1];
    let target_is_dir = target.is_dir();

    if sources.len() > 1 && !target_is_dir {
        eprintln!("cp: target '{}' is not a directory", target.display());
        std::process::exit(1);
    }

    for src_str in sources {
        let src = Path::new(src_str);
        let dst = if target_is_dir {
            target.join(src.file_name().unwrap_or_default())
        } else {
            target.to_path_buf()
        };

        if dst.exists() && !force {
            if interactive {
                eprint!("cp: overwrite '{}'? ", dst.display());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if !input.trim().eq_ignore_ascii_case("y") { continue; }
            } else if !force {
                eprintln!("cp: '{}' exists (use -f to force)", dst.display());
                std::process::exit(1);
            }
        }

        if src.is_dir() {
            if let Err(e) = copy_dir(src, &dst, recursive, preserve, interactive, force) {
                eprintln!("cp: cannot copy '{}': {}", src.display(), e);
                std::process::exit(1);
            }
        } else {
            if let Err(e) = copy_file(src, &dst, preserve) {
                eprintln!("cp: cannot copy '{}': {}", src.display(), e);
                std::process::exit(1);
            }
        }
    }
}
