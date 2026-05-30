fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: logger <message>");
        std::process::exit(1);
    }
    let msg = args[1..].join(" ");
    #[cfg(unix)]
    {
        let c_msg = std::ffi::CString::new(msg.clone()).unwrap_or_default();
        unsafe {
            libc::syslog(libc::LOG_USER | libc::LOG_INFO, "%s\0".as_ptr() as *const libc::c_char, c_msg.as_ptr());
        }
    }
    eprintln!("{}", msg);
}
