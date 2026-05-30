fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: newgrp [-] [group]");
        std::process::exit(1);
    }
    let group = if args[1] == "-" {
        if args.len() > 2 { &args[2] } else { return; }
    } else {
        &args[1]
    };
    let gid = match group_name_to_gid(group) {
        Some(id) => id,
        None => {
            eprintln!("newgrp: {}: unknown group", group);
            std::process::exit(1);
        }
    };
    let ret = unsafe { libc::setregid(gid, gid) };
    if ret != 0 {
        eprintln!("newgrp: cannot change group to {}: {:?}", group, std::io::Error::last_os_error());
        std::process::exit(1);
    }
    if let Some(shell) = std::env::var("SHELL").ok() {
        let _ = std::process::Command::new(&shell).spawn();
    }
}

fn group_name_to_gid(name: &str) -> Option<u32> {
    unsafe {
        let bufsize = libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) as usize;
        let mut buf = vec![0u8; bufsize.max(1024)];
        let mut grp: libc::group = std::mem::zeroed();
        let mut result: *mut libc::group = std::ptr::null_mut();
        let cname = std::ffi::CString::new(name).ok()?;
        let ret = libc::getgrnam_r(cname.as_ptr(), &mut grp, buf.as_mut_ptr() as *mut libc::c_char, buf.len(), &mut result);
        if ret == 0 && !result.is_null() {
            Some(grp.gr_gid)
        } else {
            None
        }
    }
}
