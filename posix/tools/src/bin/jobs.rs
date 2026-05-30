fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flags: Vec<&str> = args[1..].iter().map(|s| s.as_str()).filter(|s| s.starts_with('-')).collect();
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let jobs_file = format!("/tmp/jobs_{}", user);
    if jobs_file == "/tmp/jobs_unknown" {
        return;
    }
    let content = match std::fs::read_to_string(&jobs_file) {
        Ok(c) => c,
        Err(_) => return,
    };
    if flags.contains(&"-p") || flags.contains(&"-l") {
        // -p: print PID only, -l: print PID + command
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let rest = parts[1];
                let pid_str = rest.split_whitespace().next().unwrap_or("");
                if flags.contains(&"-p") {
                    println!("{}", pid_str);
                } else {
                    let cmd = rest.splitn(2, ' ').nth(1).unwrap_or("");
                    let status = get_status(pid_str);
                    println!("[{}] {} {} {}", parts[0], status, pid_str, cmd);
                }
            }
        }
    } else {
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let rest = parts[1];
                let pid_str = rest.split_whitespace().next().unwrap_or("");
                let cmd = rest.splitn(2, ' ').nth(1).unwrap_or("");
                let status = get_status(pid_str);
                println!("[{}] {} {}", parts[0], status, cmd);
            }
        }
    }
}

fn get_status(pid_str: &str) -> String {
    if let Ok(pid) = pid_str.parse::<i32>() {
        let ret = unsafe { libc::kill(pid, 0) };
        if ret == 0 {
            if is_stopped(pid) {
                "Stopped".to_string()
            } else {
                "Running".to_string()
            }
        } else {
            "Done".to_string()
        }
    } else {
        "Unknown".to_string()
    }
}

fn is_stopped(pid: i32) -> bool {
    let mut status: i32 = 0;
    let ret = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG | libc::WUNTRACED) };
    if ret > 0 && libc::WIFSTOPPED(status) {
        return true;
    }
    false
}
