fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pid_str = parse_job(&args, 1);
    let pid: i32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("fg: invalid argument: {}", pid_str);
            std::process::exit(1);
        }
    };
    // Send SIGCONT
    let ret = unsafe { libc::kill(pid, libc::SIGCONT) };
    if ret != 0 {
        eprintln!("fg: {}: {}", pid, std::io::Error::last_os_error());
        std::process::exit(1);
    }
    // Try to bring to foreground
    unsafe {
        libc::tcsetpgrp(libc::STDIN_FILENO, pid);
    }
    // Wait for the process
    let mut status: i32 = 0;
    unsafe {
        libc::waitpid(pid, &mut status, 0);
    }
}

fn parse_job(args: &[String], idx: usize) -> String {
    if idx >= args.len() {
        eprintln!("Usage: fg [%jobspec | pid]");
        std::process::exit(1);
    }
    let s = &args[idx];
    if s.starts_with('%') {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let jobs_file = format!("/tmp/jobs_{}", user);
        if let Ok(content) = std::fs::read_to_string(&jobs_file) {
            for line in content.lines() {
                let prefix = format!("{} ", &s[1..]);
                if let Some(rest) = line.strip_prefix(&prefix) {
                    if let Some(pid_str) = rest.split_whitespace().next() {
                        return pid_str.to_string();
                    }
                }
            }
        }
        eprintln!("fg: {}: no such job", s);
        std::process::exit(1);
    }
    s.clone()
}
