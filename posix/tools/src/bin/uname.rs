fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Use libc::uname on supported platforms, fallback otherwise
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    let uts = {
        let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
        unsafe { libc::uname(&mut uts); }
        uts
    };

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd")))]
    let uts = {
        let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
        uts
    };

    let sysname = cstr(&uts.sysname);
    let nodename = cstr(&uts.nodename);
    let release = cstr(&uts.release);
    let version = cstr(&uts.version);
    let machine = cstr(&uts.machine);

    let mut all = args.len() == 1;
    let mut show_sysname = false;
    let mut show_nodename = false;
    let mut show_release = false;
    let mut show_version = false;
    let mut show_machine = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "-a" => { all = true; }
            "-s" | "--kernel-name" => { show_sysname = true; }
            "-n" | "--nodename" => { show_nodename = true; }
            "-r" | "--kernel-release" => { show_release = true; }
            "-v" | "--kernel-version" => { show_version = true; }
            "-m" | "--machine" => { show_machine = true; }
            _ => {
                eprintln!("uname: invalid option -- '{}'", arg);
                std::process::exit(1);
            }
        }
    }

    if args.len() == 1 {
        // Default (no flags): print just sysname
        println!("{}", sysname);
        return;
    }

    let mut parts = Vec::new();
    if all || show_sysname { parts.push(sysname); }
    if all || show_nodename { parts.push(nodename); }
    if all || show_release { parts.push(release); }
    if all || show_version { parts.push(version); }
    if all || show_machine { parts.push(machine); }

    println!("{}", parts.join(" "));
}

fn cstr(buf: &[i8]) -> &str {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    // SAFETY: libc::utsname fields are null-terminated C strings
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, end) };
    std::str::from_utf8(bytes).unwrap_or("")
}
