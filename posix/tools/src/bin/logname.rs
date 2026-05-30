fn main() {
    #[cfg(unix)]
    unsafe {
        let name = libc::getlogin();
        if !name.is_null() {
            let cstr = std::ffi::CStr::from_ptr(name);
            let s = cstr.to_string_lossy();
            if !s.is_empty() {
                println!("{}", s);
                return;
            }
        }
    }
    let name = std::env::var("LOGNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string());
    println!("{}", name);
}
