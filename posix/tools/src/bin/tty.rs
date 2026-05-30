fn main() {
    match std::env::var("TERM") {
        Ok(_) => {
            #[cfg(unix)]
            unsafe {
                let fd = libc::open("/dev/tty\0".as_ptr() as *const libc::c_char, libc::O_RDONLY);
                if fd >= 0 {
                    let mut buf = [0i8; 1024];
                    if libc::ttyname_r(fd, buf.as_mut_ptr(), buf.len()) == 0 {
                        let cstr = std::ffi::CStr::from_ptr(buf.as_ptr());
                        println!("{}", cstr.to_string_lossy());
                        libc::close(fd);
                        return;
                    }
                    libc::close(fd);
                }
            }
            #[cfg(not(unix))]
            println!("/dev/tty");
        }
        Err(_) => {
            println!("not a tty");
            std::process::exit(1);
        }
    }
}
