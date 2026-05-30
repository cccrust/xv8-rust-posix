fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wait <pid>...");
        std::process::exit(1);
    }
    for pid_str in &args[1..] {
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("wait: invalid pid: {}", pid_str);
                std::process::exit(1);
            }
        };
        #[cfg(unix)]
        unsafe {
            let mut status = 0i32;
            let ret = libc::waitpid(pid as i32, &mut status as *mut i32, 0);
            if ret < 0 {
                eprintln!("wait: {}: {}", pid, std::io::Error::last_os_error());
                std::process::exit(1);
            }
        }
    }
}
