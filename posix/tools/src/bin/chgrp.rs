fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: chgrp <group> <file>...");
        std::process::exit(1);
    }
    let group_spec = &args[1];
    let files = &args[2..];

    let gid = group_to_gid(group_spec);
    match gid {
        Ok(gid) => {
            for file in files {
                #[cfg(unix)]
                {
                    let c_path = std::ffi::CString::new(file.as_str()).unwrap_or_default();
                    let ret = unsafe { libc::chown(c_path.as_ptr(), !0, gid) };
                    if ret != 0 {
                        eprintln!("chgrp: {}: {}", file, std::io::Error::last_os_error());
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("chgrp: invalid group: {}", group_spec);
            std::process::exit(1);
        }
    }
}

fn group_to_gid(name: &str) -> Result<u32, ()> {
    if let Ok(n) = name.parse::<u32>() {
        return Ok(n);
    }
    #[cfg(unix)]
    unsafe {
        let c_name = std::ffi::CString::new(name).unwrap_or_default();
        let gr = libc::getgrnam(c_name.as_ptr());
        if !gr.is_null() {
            return Ok((*gr).gr_gid);
        }
    }
    Err(())
}
