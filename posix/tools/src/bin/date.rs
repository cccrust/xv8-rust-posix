use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut utc = false;
    let mut format = String::new();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'u' => utc = true,
                _ => { eprintln!("date: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i < args.len() && args[i].starts_with('+') {
        format = args[i][1..].to_string();
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    #[cfg(unix)]
    {
        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            let t = now as libc::time_t;
            if utc {
                libc::gmtime_r(&t, &mut tm);
            } else {
                libc::localtime_r(&t, &mut tm);
            }
            let y = tm.tm_year + 1900;
            let m = tm.tm_mon + 1;
            let d = tm.tm_mday;
            let h = tm.tm_hour;
            let min = tm.tm_min;
            let sec = tm.tm_sec;

            if format.is_empty() {
                const MONTHS: &[&str] = &["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
                const DAYS: &[&str] = &["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];
                println!("{} {} {:2} {:02}:{:02}:{:02} {} {}", DAYS[tm.tm_wday as usize], MONTHS[tm.tm_mon as usize], d, h, min, sec, y, if utc { "UTC" } else { "" });
            } else {
                emit_format(&format, y, m as u32, d as u32, h as u32, min as u32, sec as u32);
            }
            return;
        }
    }

    #[cfg(not(unix))]
    {
        // Fallback: calculate from epoch
        let days = now / 86400;
        let time_secs = now % 86400;
        let hour = time_secs / 3600;
        let minute = (time_secs % 3600) / 60;
        let sec = time_secs % 60;

        if format.is_empty() {
            println!("{}", now);
        } else {
            let mut days_remaining = days;
            let mut year = 1970i64;
            loop {
                let diy = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
                if days_remaining < diy { break; }
                days_remaining -= diy;
                year += 1;
            }
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            let month_days: [i64; 12] = if leap { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
            let mut month = 1u32;
            for (i, &md) in month_days.iter().enumerate() {
                if days_remaining < md { month = (i + 1) as u32; break; }
                days_remaining -= md;
            }
            emit_format(&format, year, month, (days_remaining + 1) as u32, hour as u32, minute as u32, sec as u32);
        }
    }
}

fn emit_format(fmt: &str, y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) {
    let mut out = fmt.to_string();
    out = out.replace("%Y", &format!("{:04}", y));
    out = out.replace("%m", &format!("{:02}", m));
    out = out.replace("%d", &format!("{:02}", d));
    out = out.replace("%H", &format!("{:02}", h));
    out = out.replace("%M", &format!("{:02}", min));
    out = out.replace("%S", &format!("{:02}", s));
    println!("{}", out);
}
