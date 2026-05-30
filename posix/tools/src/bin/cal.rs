fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Default: print current month calendar
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    #[cfg(unix)]
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let t = now as libc::time_t;
        libc::localtime_r(&t, &mut tm);
        let year = tm.tm_year + 1900;
        let month = tm.tm_mon + 1;
        if args.len() > 1 {
            if let Ok(y) = args[1].parse::<i32>() {
                print_year(y);
                return;
            }
        }
        print_month(month, year);
    }
}

fn print_month(month: i32, year: i32) {
    let months = ["January","February","March","April","May","June","July","August","September","October","November","December"];
    let m = month.max(1).min(12) as usize;
    let title = format!("{} {}", months[m-1], year);
    println!("{:^20}", title);
    println!("Su Mo Tu We Th Fr Sa");

    let first = weekday(1, month, year);
    let days = days_in_month(month, year);
    for _ in 0..first { print!("   "); }
    for d in 1..=days {
        print!("{:2} ", d);
        if (d as usize + first) % 7 == 0 { println!(); }
    }
    if (days as usize + first) % 7 != 0 { println!(); }
}

fn print_year(year: i32) {
    for m in 1..=12 {
        print_month(m, year);
        println!();
    }
}

fn weekday(day: i32, month: i32, year: i32) -> usize {
    // Zeller-like for Gregorian
    let (m, y) = if month < 3 { (month + 12, year - 1) } else { (month, year) };
    ((day + (13 * (m + 1)) / 5 + y % 100 + (y % 100) / 4 + (y / 100) / 4 - 2 * (y / 100)) % 7 + 7) as usize % 7
}

fn days_in_month(month: i32, year: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(year) { 29 } else { 28 },
        _ => 0,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
