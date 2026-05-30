use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut recursive = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'R' => recursive = true,
                _ => { eprintln!("chown: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i + 1 >= args.len() {
        eprintln!("usage: chown [-R] owner[:group] file ...");
        std::process::exit(1);
    }

    let owner_str = &args[i];
    i += 1;

    let (uid_str, gid_str) = if let Some(at) = owner_str.find(':') {
        (&owner_str[..at], Some(&owner_str[at + 1..]))
    } else {
        (owner_str.as_str(), None)
    };

    #[cfg(unix)]
    let uid = if uid_str.is_empty() { !0u32 } else {
        if let Ok(n) = uid_str.parse::<u32>() { n }
        else {
            unsafe {
                let pw = libc::getpwnam(std::ffi::CString::new(uid_str.as_bytes()).unwrap_or_default().as_ptr());
                if !pw.is_null() { (*pw).pw_uid } else { !0u32 }
            }
        }
    };

    #[cfg(unix)]
    let gid = gid_str.and_then(|s| {
        if s.is_empty() { None }
        else if let Ok(n) = s.parse::<u32>() { Some(n) }
        else {
            unsafe {
                let gr = libc::getgrnam(std::ffi::CString::new(s.as_bytes()).unwrap_or_default().as_ptr());
                if !gr.is_null() { Some((*gr).gr_gid) } else { None }
            }
        }
    });

    #[cfg(not(unix))]
    { eprintln!("chown: not supported on this platform"); std::process::exit(1); }

    fn chown_rec(path: &Path, uid: u32, gid: Option<u32>, recursive: bool) {
        #[cfg(unix)]
        {
            let path_c = std::ffi::CString::new(path.to_str().unwrap_or("")).unwrap_or_default();
            let gid_val = gid.unwrap_or(!0u32);
            unsafe { libc::chown(path_c.as_ptr(), uid, gid_val); }
        }
        if recursive && path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    chown_rec(&entry.path(), uid, gid, true);
                }
            }
        }
    }

    for path_str in &args[i..] {
        let path = Path::new(path_str);
        #[cfg(unix)]
        chown_rec(path, uid, gid, recursive);
    }
}
