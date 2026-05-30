use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: write <user> [tty]");
        std::process::exit(1);
    }
    let user = &args[1];
    let tty = if args.len() > 2 { Some(args[2].as_str()) } else { None };

    // Find the target user's terminal
    match find_terminal(user, tty) {
        Some(path) => {
            let mut f = match std::fs::OpenOptions::new().write(true).open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("write: {}: {}", path, e);
                    std::process::exit(1);
                }
            };
            let caller = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            writeln!(f, "Message from {}@{} to {} on {}...", caller, hostname(), user, path).ok();
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap_or_default();
                if line.trim() == "EOF" { break; }
                writeln!(f, "{}", line).ok();
            }
            writeln!(f, "EOF").ok();
        }
        None => {
            eprintln!("write: {} is not logged in", user);
            std::process::exit(1);
        }
    }
}

fn hostname() -> String {
    #[cfg(unix)]
    unsafe {
        let mut buf = [0i8; 256];
        if libc::gethostname(buf.as_mut_ptr(), buf.len()) == 0 {
            return std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string();
        }
    }
    "localhost".to_string()
}

fn find_terminal(user: &str, tty: Option<&str>) -> Option<String> {
    // Check /dev/pts/* on Linux or /dev/ttys* on macOS
    let dev = std::path::Path::new("/dev");
    if let Ok(entries) = std::fs::read_dir(dev) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if tty.map_or(true, |t| name == t || name.ends_with(t)) {
                // Check ownership
                if let Ok(meta) = entry.metadata() {
                    #[cfg(unix)]
                    use std::os::unix::fs::MetadataExt;
                    let owner = meta.uid();
                    if let Ok(passwd) = user_to_uid(user) {
                        if owner == passwd {
                            return Some(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn user_to_uid(user: &str) -> Result<u32, ()> {
    #[cfg(unix)]
    unsafe {
        let c_user = std::ffi::CString::new(user).unwrap_or_default();
        let pw = libc::getpwnam(c_user.as_ptr());
        if !pw.is_null() {
            return Ok((*pw).pw_uid);
        }
    }
    Err(())
}
