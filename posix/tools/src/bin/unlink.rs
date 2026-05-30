fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: unlink <file>");
        std::process::exit(1);
    }
    #[cfg(unix)]
    {
        let c_path = std::ffi::CString::new(args[1].as_str()).unwrap_or_default();
        let ret = unsafe { libc::unlink(c_path.as_ptr()) };
        if ret != 0 {
            eprintln!("unlink: {}: {}", args[1], std::io::Error::last_os_error());
            std::process::exit(1);
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("unlink: not supported on this platform");
        std::process::exit(1);
    }
}
