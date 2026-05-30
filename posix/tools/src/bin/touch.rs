use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn parse_time(s: &str) -> Option<SystemTime> {
    let s = s.trim();
    let parts: Vec<&str> = if s.contains('.') {
        s.splitn(2, '.').collect()
    } else {
        vec![s, "00"]
    };
    let datetime = parts[0];
    let sec_str = parts.get(1).unwrap_or(&"00");
    let second: u32 = sec_str.parse().unwrap_or(0);

    let len = datetime.len();
    let (year, month, day, hour, minute) = if len == 8 {
        // MMDDhhmm - use current year
        let month: u32 = datetime[0..2].parse().ok()?;
        let day: u32 = datetime[2..4].parse().ok()?;
        let hour: u32 = datetime[4..6].parse().ok()?;
        let minute: u32 = datetime[6..8].parse().ok()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
        let days = now.as_secs() / 86400;
        let mut y = 1970i32;
        let mut d = days as i64;
        loop {
            let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
            if d < days_in_year { break; }
            d -= days_in_year;
            y += 1;
        }
        (y, month, day, hour, minute)
    } else if len == 10 {
        let year: i32 = datetime[0..4].parse().ok()?;
        let month: u32 = datetime[4..6].parse().ok()?;
        let day: u32 = datetime[6..8].parse().ok()?;
        let hour: u32 = datetime[8..10].parse().ok()?;
        (year, month, day, hour, 0u32)
    } else if len == 12 {
        let year: i32 = datetime[0..4].parse().ok()?;
        let month: u32 = datetime[4..6].parse().ok()?;
        let day: u32 = datetime[6..8].parse().ok()?;
        let hour: u32 = datetime[8..10].parse().ok()?;
        let minute: u32 = datetime[10..12].parse().ok()?;
        (year, month, day, hour, minute)
    } else {
        return None;
    };

    let mut days_from_epoch = 0i64;
    for y in 1970..year {
        days_from_epoch += if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
    }
    let month_days: [u64; 12] = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
    for m in 0..(month as usize - 1) {
        days_from_epoch += month_days[m] as i64;
    }
    days_from_epoch += (day as i64) - 1;
    let total_secs = days_from_epoch * 86400 + (hour as i64) * 3600 + (minute as i64) * 60 + second as i64;
    if total_secs >= 0 {
        Some(UNIX_EPOCH + std::time::Duration::from_secs(total_secs as u64))
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut no_create = false;
    let mut ref_file: Option<String> = None;
    let mut time_str_opt: Option<String> = None;
    let mut set_access = false;
    let mut set_modify = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'a' => set_access = true,
                'c' => no_create = true,
                'm' => set_modify = true,
                'r' => { i += 1; if i < args.len() { ref_file = Some(args[i].clone()); } }
                't' => { i += 1; if i < args.len() { time_str_opt = Some(args[i].clone()); } }
                _ => { eprintln!("touch: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: touch [-acm] [-r ref] [-t time] file ...");
        std::process::exit(1);
    }

    let ref_time = ref_file.as_ref().and_then(|p| fs::metadata(p).ok().map(|m| {
        (m.accessed().unwrap_or(UNIX_EPOCH), m.modified().unwrap_or(UNIX_EPOCH))
    }));

    let specified_time = time_str_opt.as_ref().and_then(|s| parse_time(s));
    let now = SystemTime::now();

    for path_str in &args[i..] {
        let path = Path::new(path_str);
        let exists = path.exists();

        if !exists {
            if no_create { continue; }
            if let Err(e) = fs::write(path, "") {
                eprintln!("touch: cannot create '{}': {}", path.display(), e);
                std::process::exit(1);
            }
        }

        let new_atime = if set_access || (!set_access && !set_modify) {
            ref_time.map(|(a, _)| a).or(specified_time).unwrap_or(now)
        } else {
            fs::metadata(path).ok().and_then(|m| m.accessed().ok()).unwrap_or(now)
        };
        let new_mtime = if set_modify || (!set_access && !set_modify) {
            ref_time.map(|(_, m)| m).or(specified_time).unwrap_or(now)
        } else {
            fs::metadata(path).ok().and_then(|m| m.modified().ok()).unwrap_or(now)
        };

        #[cfg(unix)]
        {
            let atime = libc::timeval {
                tv_sec: new_atime.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                tv_usec: 0,
            };
            let mtime = libc::timeval {
                tv_sec: new_mtime.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                tv_usec: 0,
            };
            let path_s = path.to_str().unwrap_or("");
            let path_c = std::ffi::CString::new(path_s).unwrap_or_default();
            unsafe { libc::utimes(path_c.as_ptr(), [atime, mtime].as_ptr()); }
        }
    }
}
