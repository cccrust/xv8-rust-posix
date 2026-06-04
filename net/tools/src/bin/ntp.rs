use libnet::ntp;
use libnet::net_impl::UNIX_EPOCH;

fn format_utc(secs: u64, nsecs: u32) -> String {
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
        if d < days_in_year { break; }
        d -= days_in_year;
        y += 1;
    }
    let month_days = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0u32;
    for (i, &md) in month_days.iter().enumerate() {
        if d < md {
            m = i as u32 + 1;
            break;
        }
        d -= md;
    }
    if m == 0 {
        m = 12;
        d = 0;
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} UTC", y, m, d + 1, hours, minutes, seconds, nsecs / 1_000_000)
}

fn usage() -> ! {
    eprintln!("Usage: ntp <server>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }

    let server = &args[1];
    match ntp::query(server) {
        Ok(pkt) => {
            let sys = pkt.transmit_ts.to_system_time();
            let dur = sys.duration_since(UNIX_EPOCH).unwrap_or_default();
            let datetime = format_utc(dur.as_secs(), dur.subsec_nanos());

            println!("NTP query to {}", server);
            println!("  Stratum:      {}", pkt.stratum);
            println!("  Poll:         {}s", pkt.poll);
            println!("  Precision:    {}s", pkt.precision as i8);
            println!("  Root delay:   {:.6}s", (pkt.root_delay as f64) / 65536.0);
            println!("  Root disp:    {:.6}s", (pkt.root_dispersion as f64) / 65536.0);
            println!("  Reference ID: {:#010x}", pkt.reference_id);
            println!("  Timestamp:    {}", datetime);
        }
        Err(e) => {
            eprintln!("ntp: {}: {}", server, e);
            std::process::exit(1);
        }
    }
}
