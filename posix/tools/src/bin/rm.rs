use std::fs;
use std::path::Path;

fn remove_dir_recursive(path: &Path) -> Result<(), String> {
    let entries = fs::read_dir(path).map_err(|e| format!("{}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}", e))?;
        let p = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            remove_dir_recursive(&p)?;
        } else {
            fs::remove_file(&p).map_err(|e| format!("cannot remove '{}': {}", p.display(), e))?;
        }
    }
    fs::remove_dir(path).map_err(|e| format!("cannot remove '{}': {}", path.display(), e))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut recursive = false;
    let mut force = false;
    let mut interactive = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'r' | 'R' => recursive = true,
                'f' => force = true,
                'i' => interactive = true,
                _ => { eprintln!("rm: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: rm [-Rif] file ...");
        std::process::exit(1);
    }

    for path_str in &args[i..] {
        let path = Path::new(path_str);

        if !path.exists() {
            if force { continue; }
            eprintln!("rm: cannot remove '{}': No such file or directory", path.display());
            std::process::exit(1);
        }

        if interactive {
            let entry_type = if path.is_dir() { "directory" } else { "file" };
            eprint!("rm: remove {} '{}'? ", entry_type, path.display());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if !input.trim().eq_ignore_ascii_case("y") { continue; }
        }

        if path.is_dir() {
            if !recursive {
                eprintln!("rm: cannot remove '{}': Is a directory", path.display());
                if !force { std::process::exit(1); }
                continue;
            }
            if let Err(e) = remove_dir_recursive(path) {
                eprintln!("rm: {}", e);
                if !force { std::process::exit(1); }
            }
        } else {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("rm: cannot remove '{}': {}", path.display(), e);
                if !force { std::process::exit(1); }
            }
        }
    }
}
