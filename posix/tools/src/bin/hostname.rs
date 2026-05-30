fn main() {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| {
            #[cfg(unix)]
            {
                let mut buf = [0i8; 256];
                unsafe {
                    if libc::gethostname(buf.as_mut_ptr(), buf.len()) == 0 {
                        return std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string();
                    }
                }
            }
            "localhost".to_string()
        });

    println!("{}", hostname);
}
