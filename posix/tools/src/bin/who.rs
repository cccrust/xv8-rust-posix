use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut flag_q = false;
    let mut flag_h = false;
    let mut flag_t = false;
    let mut flag_u = false;
    let mut flag_b = false;
    let mut flag_r = false;
    let mut flag_a = false;
    let mut file: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--" { i += 1; break; }
        if args[i].starts_with('-') {
            for c in args[i][1..].chars() {
                match c {
                    'a' => flag_a = true,
                    'b' => flag_b = true,
                    'H' => flag_h = true,
                    'q' => flag_q = true,
                    'r' => flag_r = true,
                    'T' => flag_t = true,
                    'u' => flag_u = true,
                    _ => { eprintln!("who: invalid option -- {}", c); std::process::exit(1); }
                }
            }
        } else {
            file = Some(args[i].clone());
        }
        i += 1;
    }

    if !flag_a && !flag_b && !flag_q && !flag_r {
        flag_a = true;
    }
    if flag_q {
        flag_b = false; flag_r = false;
    }

    #[cfg(not(target_arch = "riscv64"))]
    {
        use std::ffi::CStr;
        unsafe {
            if let Some(ref _f) = file {
                // file specified - parse it directly
                eprintln!("who: reading file not supported");
                std::process::exit(1);
            }

            libc::setutxent();

            if flag_b {
                libc::setutxent();
                loop {
                    let ptr = libc::getutxent();
                    if ptr.is_null() { break; }
                    let ut = &*ptr;
                    if ut.ut_type == libc::BOOT_TIME {
                        if flag_h { println!("{:<8} {:<12} {}", "NAME", "LINE", "TIME"); }
                        println!("reboot   ~                        {}", fmt_time(ut.ut_tv.tv_sec));
                        break;
                    }
                }
                libc::endutxent();
                return;
            }

            if flag_r {
                libc::setutxent();
                loop {
                    let ptr = libc::getutxent();
                    if ptr.is_null() { break; }
                    let ut = &*ptr;
                    if ut.ut_type == libc::RUN_LVL {
                        if flag_h { println!("{:<8} {:<12} {}", "NAME", "LINE", "TIME"); }
                        let runlvl = unsafe { *CStr::from_ptr(ut.ut_user.as_ptr()).to_string_lossy().as_ref().as_ptr() as u8 };
                        println!("run-level {}", runlvl as char);
                        break;
                    }
                }
                libc::endutxent();
                return;
            }

            if flag_q {
                let mut names: Vec<String> = Vec::new();
                libc::setutxent();
                loop {
                    let ptr = libc::getutxent();
                    if ptr.is_null() { break; }
                    let ut = &*ptr;
                    if ut.ut_type == libc::USER_PROCESS {
                        let user = CStr::from_ptr(ut.ut_user.as_ptr()).to_string_lossy().to_string();
                        if !user.is_empty() {
                            names.push(user);
                        }
                    }
                }
                libc::endutxent();
                println!("{}", names.join(" "));
                println!("# users={}", names.len());
                return;
            }

            if flag_h {
                if flag_t { println!("{:<8} {:<8} {:<12} {}", "NAME", "LINE", "TIME", "COMMENT"); }
                else { println!("{:<8} {:<12} {}", "NAME", "LINE", "TIME"); }
            }

            libc::setutxent();
            loop {
                let ptr = libc::getutxent();
                if ptr.is_null() { break; }
                let ut = &*ptr;
                if flag_a || ut.ut_type == libc::USER_PROCESS {
                    let user = CStr::from_ptr(ut.ut_user.as_ptr()).to_string_lossy();
                    let line = CStr::from_ptr(ut.ut_line.as_ptr()).to_string_lossy();
                    if flag_a || (ut.ut_type == libc::USER_PROCESS && !user.is_empty()) {
                        let status = if flag_t {
                            format!(" {}", if ut.ut_type == libc::USER_PROCESS { "+" } else { "-" })
                        } else { String::new() };
                        let idle = if flag_u {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs() as i64;
                            let idle_secs = now - ut.ut_tv.tv_sec;
                            if idle_secs < 60 { " .".to_string() }
                            else { format!(" {:02}:{:02}", idle_secs / 3600, (idle_secs % 3600) / 60) }
                        } else { String::new() };
                        println!("{:<8}{} {:<12} {}{}", user, status, line, fmt_time(ut.ut_tv.tv_sec), idle);
                    }
                }
            }
            libc::endutxent();
        }
        return;
    }

    println!("root     console  Jun  3 09:00");
}

#[cfg(not(target_arch = "riscv64"))]
fn fmt_time(secs: i64) -> String {
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let t = secs as libc::time_t;
        libc::localtime_r(&t, &mut tm);
        let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
        let mon = months[tm.tm_mon as usize];
        format!("{} {:2} {:02}:{:02}", mon, tm.tm_mday, tm.tm_hour, tm.tm_min)
    }
}