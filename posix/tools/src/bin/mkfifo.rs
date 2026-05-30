fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mkfifo <name>...");
        std::process::exit(1);
    }
    for path in &args[1..] {
        let c_path = std::ffi::CString::new(path.as_str()).unwrap_or_default();
        let ret = unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };
        if ret != 0 {
            eprintln!("mkfifo: {}: {}", path, std::io::Error::last_os_error());
            std::process::exit(1);
        }
    }
}
