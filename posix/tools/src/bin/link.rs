fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: link <existing> <new>");
        std::process::exit(1);
    }
    #[cfg(unix)]
    {
        let c_existing = std::ffi::CString::new(args[1].as_str()).unwrap_or_default();
        let c_new = std::ffi::CString::new(args[2].as_str()).unwrap_or_default();
        let ret = unsafe { libc::link(c_existing.as_ptr(), c_new.as_ptr()) };
        if ret != 0 {
            eprintln!("link: {}: {}", args[2], std::io::Error::last_os_error());
            std::process::exit(1);
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("link: not supported on this platform");
        std::process::exit(1);
    }
}
