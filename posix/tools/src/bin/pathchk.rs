use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: pathchk <path>...");
        std::process::exit(1);
    }
    let mut ok = true;
    for path in &args[1..] {
        if !check_path(path) {
            ok = false;
        }
    }
    if !ok {
        std::process::exit(1);
    }
}

fn check_path(path: &str) -> bool {
    if path.is_empty() {
        eprintln!("pathchk: empty pathname");
        return false;
    }
    if path.len() > 255 {
        eprintln!("pathchk: {}: pathname too long (max 255)", path);
        return false;
    }
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            let parent_str = parent.to_string_lossy();
            if parent_str.len() > 255 {
                eprintln!("pathchk: {}: pathname too long", path);
                return false;
            }
            if let Some(dirname) = parent.file_name() {
                let d = dirname.to_string_lossy();
                if d.len() > 255 {
                    eprintln!("pathchk: {}: pathname too long", path);
                    return false;
                }
            }
        }
    }
    if let Some(filename) = p.file_name() {
        let f = filename.to_string_lossy();
        if f.len() > 255 {
            eprintln!("pathchk: {}: filename too long (max 255)", path);
            return false;
        }
        if f.contains('/') {
            eprintln!("pathchk: {}: contains '/' in filename", path);
            return false;
        }
    }
    true
}
