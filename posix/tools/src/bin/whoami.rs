fn main() {
    let name = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| {
            // On Unix, fall back to getpwuid(geteuid())
            #[cfg(unix)]
            unsafe {
                let uid = libc::geteuid();
                let pw = libc::getpwuid(uid);
                if !pw.is_null() {
                    return std::ffi::CStr::from_ptr((*pw).pw_name).to_string_lossy().to_string();
                }
            }
            "unknown".to_string()
        });
    println!("{}", name);
}
