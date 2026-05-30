fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: renice [-n <increment>] <pid>...");
        std::process::exit(1);
    }
    let mut i = 1;
    let mut increment = 0i32;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-n" if i + 1 < args.len() => {
                increment = args[i + 1].parse().unwrap_or(0);
                i += 2;
            }
            _ => { i += 1; }
        }
    }
    if i >= args.len() {
        eprintln!("Usage: renice [-n <increment>] <pid>...");
        std::process::exit(1);
    }
    for pid_str in &args[i..] {
        let pid: i32 = pid_str.parse().unwrap_or(0);
        #[cfg(unix)]
        unsafe {
            let ret = libc::setpriority(libc::PRIO_PROCESS, pid as u32, increment);
            if ret != 0 {
                eprintln!("renice: {}: {}", pid, std::io::Error::last_os_error());
            } else {
                let old = libc::getpriority(libc::PRIO_PROCESS, pid as u32);
                eprintln!("{}: old priority {}", pid, old + increment);
            }
        }
    }
}
