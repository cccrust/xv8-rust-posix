#[cfg(unix)]
fn get_ids() -> (u32, u32, u32, u32) {
    unsafe {
        (libc::getuid(), libc::geteuid(), libc::getgid(), libc::getegid())
    }
}

#[cfg(not(unix))]
fn get_ids() -> (u32, u32, u32, u32) {
    (0, 0, 0, 0)
}

#[cfg(unix)]
fn user_name(uid: u32) -> String {
    unsafe {
        let pw = libc::getpwuid(uid);
        if !pw.is_null() {
            return std::ffi::CStr::from_ptr((*pw).pw_name).to_string_lossy().to_string();
        }
    }
    uid.to_string()
}

#[cfg(not(unix))]
fn user_name(uid: u32) -> String {
    uid.to_string()
}

#[cfg(unix)]
fn group_name(gid: u32) -> String {
    unsafe {
        let gr = libc::getgrgid(gid);
        if !gr.is_null() {
            return std::ffi::CStr::from_ptr((*gr).gr_name).to_string_lossy().to_string();
        }
    }
    gid.to_string()
}

#[cfg(not(unix))]
fn group_name(gid: u32) -> String {
    gid.to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (ruid, euid, rgid, egid) = get_ids();

    let mut show_groups = false;
    let mut show_group = false;
    let mut show_name = false;
    let mut show_real = false;
    let mut show_user = false;

    // Parse options (handle combined short options like -ur)
    let mut opts_remaining: Vec<char> = Vec::new();
    for arg in &args[1..] {
        if arg.starts_with('-') && arg.len() > 1 && arg.as_bytes()[1] != b'-' {
            for c in arg[1..].chars() {
                opts_remaining.push(c);
            }
        } else {
            // Stop at first non-option
            break;
        }
    }

    for c in opts_remaining {
        match c {
            'G' => show_groups = true,
            'g' => show_group = true,
            'n' => show_name = true,
            'r' => show_real = true,
            'u' => show_user = true,
            _ => {
                eprintln!("id: invalid option -- '{}'", c);
                std::process::exit(1);
            }
        }
    }

    let use_real = show_real;
    let uid = if use_real { ruid } else { euid };
    let gid = if use_real { rgid } else { egid };

    let format_id = |id: u32| -> String {
        if show_name { user_name(id) } else { id.to_string() }
    };

    if show_groups {
        #[cfg(unix)]
        unsafe {
            let ngroup = libc::getgroups(0, std::ptr::null_mut());
            if ngroup > 0 {
                let mut groups: Vec<libc::gid_t> = vec![0; ngroup as usize];
                libc::getgroups(ngroup, groups.as_mut_ptr());
                let names: Vec<String> = groups.iter().map(|g| format_id(*g as u32)).collect();
                println!("{}", names.join(" "));
                return;
            }
        }
        println!("{}", format_id(gid));
        return;
    }

    if show_group {
        println!("{}", format_id(gid));
        return;
    }

    if show_user {
        println!("{}", format_id(uid));
        return;
    }

    // Default: print full info
    println!("uid={}({}) gid={}({})", uid, user_name(uid), gid, group_name(gid));
}
