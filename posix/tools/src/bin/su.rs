fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut user = String::new();
    let mut cmd: Vec<String> = Vec::new();
    let mut is_login = false;

    while i < args.len() && args[i].starts_with('-') {
        if args[i] == "-" || args[i] == "-l" || args[i] == "--login" {
            is_login = true;
        } else if args[i] == "-c" {
            // -c command: collect remaining args as command
            i += 1;
            while i < args.len() {
                cmd.push(args[i].clone());
                i += 1;
            }
            break;
        }
        i += 1;
    }

    if i < args.len() && cmd.is_empty() {
        user = args[i].clone();
        i += 1;
        if i < args.len() && args[i] == "-c" {
            i += 1;
            while i < args.len() {
                cmd.push(args[i].clone());
                i += 1;
            }
        }
    }

    if cmd.is_empty() {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        if !user.is_empty() {
            let uid = lookup_user(&user);
            if let Some(uid) = uid {
                if unsafe { libc::setuid(uid) } != 0 {
                    eprintln!("su: permission denied");
                    std::process::exit(1);
                }
            } else {
                eprintln!("su: unknown user: {}", user);
                std::process::exit(1);
            }
        }
        let mut child = std::process::Command::new(&shell);
        if is_login {
            child.arg("-l");
        }
        let status = child.status().unwrap_or_else(|e| {
            eprintln!("su: cannot execute {}: {}", shell, e);
            std::process::exit(1);
        });
        std::process::exit(status.code().unwrap_or(0));
    }

    // -c mode: run command
    if !user.is_empty() {
        let uid = lookup_user(&user);
        if let Some(uid) = uid {
            if unsafe { libc::setuid(uid) } != 0 {
                eprintln!("su: permission denied");
                std::process::exit(1);
            }
        } else {
            eprintln!("su: unknown user: {}", user);
            std::process::exit(1);
        }
    }
    let mut child = std::process::Command::new(&cmd[0]);
    child.args(&cmd[1..]);
    let status = child.status().unwrap_or_else(|e| {
        eprintln!("su: cannot execute {}: {}", cmd[0], e);
        std::process::exit(1);
    });
    std::process::exit(status.code().unwrap_or(0));
}

fn lookup_user(name: &str) -> Option<libc::uid_t> {
    unsafe {
        let pw = libc::getpwnam(
            std::ffi::CString::new(name).ok()?.as_ptr(),
        );
        if pw.is_null() {
            None
        } else {
            Some((*pw).pw_uid)
        }
    }
}
