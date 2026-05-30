fn main() {
    #[cfg(unix)]
    {
        unsafe {
            let _ut: libc::utmpx = std::mem::zeroed();
            libc::setutxent();
            while libc::getutxent() as *mut libc::utmpx != std::ptr::null_mut() {
                let ut = &*(libc::getutxent() as *const libc::utmpx);
                if ut.ut_type == libc::USER_PROCESS {
                    let user = std::ffi::CStr::from_ptr(ut.ut_user.as_ptr()).to_string_lossy();
                    let tty = std::ffi::CStr::from_ptr(ut.ut_line.as_ptr()).to_string_lossy();
                    let time = ut.ut_tv.tv_sec;
                    let date = format_time(time);
                    println!("{:<8} {:<12} {}", user, tty, date);
                }
            }
            libc::endutxent();
        }
    }
    #[cfg(not(unix))]
    {
        println!("root     console  Jan 1 00:00");
    }
}

fn format_time(secs: i64) -> String {
    #[cfg(unix)]
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let t = secs as libc::time_t;
        libc::localtime_r(&t, &mut tm);
        let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
        let mon = months[tm.tm_mon as usize];
        format!("{} {:2} {:02}:{:02}", mon, tm.tm_mday, tm.tm_hour, tm.tm_min)
    }
}
