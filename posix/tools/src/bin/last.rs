fn main() {
    #[cfg(unix)]
    unsafe {
        use std::ffi::CStr;
        let mut entries: Vec<(String, String, i64)> = Vec::new();
        libc::setutxent();
        loop {
            let ptr = libc::getutxent();
            if ptr.is_null() { break; }
            let ut = &*ptr;
            if ut.ut_type == libc::USER_PROCESS {
                let user = CStr::from_ptr(ut.ut_user.as_ptr()).to_string_lossy().to_string();
                let line = CStr::from_ptr(ut.ut_line.as_ptr()).to_string_lossy().to_string();
                if !user.is_empty() && !line.is_empty() {
                    entries.push((user, line, ut.ut_tv.tv_sec));
                }
            }
        }
        libc::endutxent();

        entries.reverse();
        let n = 20.min(entries.len());
        for i in 0..n {
            let (ref user, ref line, secs) = entries[i];
            let mut tm: libc::tm = std::mem::zeroed();
            let t = secs as libc::time_t;
            libc::localtime_r(&t, &mut tm);
            let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
            let mon = months[tm.tm_mon as usize];
            println!("{:<8} {:<12} {} {:2} {:02}:{:02}", user, line, mon, tm.tm_mday, tm.tm_hour, tm.tm_min);
        }
        return;
    }
    eprintln!("last: not supported");
}