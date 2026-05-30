fn list_signals() {
    let signals = [
        (1, "HUP"), (2, "INT"), (3, "QUIT"), (4, "ILL"), (5, "TRAP"),
        (6, "ABRT"), (7, "BUS"), (8, "FPE"), (9, "KILL"), (10, "USR1"),
        (11, "SEGV"), (12, "USR2"), (13, "PIPE"), (14, "ALRM"), (15, "TERM"),
        (16, "STKFLT"), (17, "CHLD"), (18, "CONT"), (19, "STOP"), (20, "TSTP"),
        (21, "TTIN"), (22, "TTOU"),
    ];
    for (n, name) in signals {
        print!("{:>2}) {:<4} ", n, name);
    }
    println!();
}

fn sig_from_name(s: &str) -> Option<i32> {
    let s = s.strip_prefix("SIG").unwrap_or(s);
    match s {
        "HUP" => Some(1), "INT" => Some(2), "QUIT" => Some(3),
        "ILL" => Some(4), "TRAP" => Some(5), "ABRT" => Some(6),
        "BUS" => Some(7), "FPE" => Some(8), "KILL" => Some(9),
        "USR1" => Some(10), "SEGV" => Some(11), "USR2" => Some(12),
        "PIPE" => Some(13), "ALRM" => Some(14), "TERM" => Some(15),
        "STKFLT" => Some(16), "CHLD" => Some(17), "CONT" => Some(18),
        "STOP" => Some(19), "TSTP" => Some(20), "TTIN" => Some(21),
        "TTOU" => Some(22),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut sig = 15; // SIGTERM

    if i < args.len() && args[i] == "-l" {
        list_signals();
        return;
    }

    if i < args.len() && args[i] == "-s" {
        i += 1;
        if i < args.len() {
            sig = if let Ok(n) = args[i].parse::<i32>() {
                n
            } else {
                sig_from_name(&args[i]).unwrap_or(15)
            };
            i += 1;
        }
    }

    if i < args.len() && args[i].starts_with('-') && args[i].len() > 1 {
        // -SIGNAL syntax
        let sig_str = &args[i][1..];
        sig = if let Ok(n) = sig_str.parse::<i32>() {
            n
        } else {
            sig_from_name(sig_str).unwrap_or(15)
        };
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: kill [-s signal] pid ...");
        std::process::exit(1);
    }

    for pid_str in &args[i..] {
        let pid: i32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("kill: invalid pid: {}", pid_str);
                continue;
            }
        };
        // On Unix, use libc::kill
        #[cfg(unix)]
        unsafe {
            if libc::kill(pid, sig) != 0 {
                eprintln!("kill: ({}) - {}", pid, std::io::Error::last_os_error());
                std::process::exit(1);
            }
        }
        #[cfg(not(unix))]
        {
            eprintln!("kill: not supported on this platform");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}
