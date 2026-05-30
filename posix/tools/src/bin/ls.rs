use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::UNIX_EPOCH;

const MONTHS: &[&str] = &["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

struct Config {
    all: bool, long: bool, dir: bool, classify: bool,
    inode: bool, size: bool, reverse: bool, recursive: bool,
    single: bool, comma: bool, slash_dir: bool,
}

fn mode_string(mode: u32) -> String {
    let t = match mode & 0o170000 {
        0o100000 => '-', 0o040000 => 'd', 0o120000 => 'l',
        0o020000 => 'c', 0o060000 => 'b', 0o010000 => 'p',
        0o140000 => 's', _ => '?',
    };
    let u = format!("{}{}{}",
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        match (mode & 0o100 != 0, mode & 0o4000 != 0) {
            (true, true) => 's', (false, true) => 'S', (true, false) => 'x', _ => '-',
        });
    let g = format!("{}{}{}",
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        match (mode & 0o010 != 0, mode & 0o2000 != 0) {
            (true, true) => 's', (false, true) => 'S', (true, false) => 'x', _ => '-',
        });
    let o = format!("{}{}{}",
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        match (mode & 0o001 != 0, mode & 0o1000 != 0) {
            (true, true) => 't', (false, true) => 'T', (true, false) => 'x', _ => '-',
        });
    format!("{}{}{}{}", t, u, g, o)
}

fn user_name(uid: u32) -> String {
    #[cfg(unix)]
    unsafe {
        let pw = libc::getpwuid(uid);
        if !pw.is_null() {
            return std::ffi::CStr::from_ptr((*pw).pw_name).to_string_lossy().to_string();
        }
    }
    uid.to_string()
}

fn group_name(gid: u32) -> String {
    #[cfg(unix)]
    unsafe {
        let gr = libc::getgrgid(gid);
        if !gr.is_null() {
            return std::ffi::CStr::from_ptr((*gr).gr_name).to_string_lossy().to_string();
        }
    }
    gid.to_string()
}

fn format_time(secs: i64) -> String {
    let duration = std::time::Duration::from_secs(secs.max(0) as u64);
    let base = UNIX_EPOCH + duration;
    let datetime = || -> Option<(i32, u32, u32, u32, u32, u32)> {
        let secs_since_epoch = base.duration_since(UNIX_EPOCH).ok()?.as_secs();
        let days = secs_since_epoch / 86400;
        let time_secs = secs_since_epoch % 86400;
        let h = (time_secs / 3600) as u32;
        let m = ((time_secs % 3600) / 60) as u32;
        let s = (time_secs % 60) as u32;

        let mut y = 1970i32;
        let mut d = days as i64;
        loop {
            let days_in_year = if is_leap(y) { 366 } else { 365 };
            if d < days_in_year { break; }
            d -= days_in_year;
            y += 1;
        }
        let month_days = if is_leap(y) { &LEAP_MONTH_DAYS[..] } else { &NORM_MONTH_DAYS[..] };
        let mut mo = 0u32;
        for &md in month_days {
            if d < md { break; }
            d -= md;
            mo += 1;
        }
        let day = (d + 1) as u32;
        Some((y, mo, day, h, m, s))
    }();

    let (y, mo, day, h, mi, _) = datetime.unwrap_or((1970, 0, 1, 0, 0, 0));
    let now_secs = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let is_recent = (secs - now_secs).abs() < 180 * 86400;
    if is_recent {
        format!("{} {:2} {:02}:{:02}", MONTHS[mo as usize], day, h, mi)
    } else {
        format!("{} {:2}  {:4}", MONTHS[mo as usize], day, y)
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

const NORM_MONTH_DAYS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_MONTH_DAYS: [i64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn list_long(_path: &Path, name: &str, meta: &fs::Metadata) -> String {
    let mode_bits = meta.permissions().mode();
    let mstr = mode_string(mode_bits);
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();
    let mtime = meta.mtime();
    let uname = user_name(uid);
    let gname = group_name(gid);
    let t = format_time(mtime);

    format!("{} {:>3} {:<8} {:<8} {:>8} {} {}", mstr, nlink, uname, gname, size, t, name)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut cfg = Config {
        all: false, long: false, dir: false, classify: false,
        inode: false, size: false, reverse: false, recursive: false,
        single: false, comma: false, slash_dir: false,
    };

    let mut paths: Vec<String> = Vec::new();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'a' => cfg.all = true, 'l' => cfg.long = true, 'd' => cfg.dir = true,
                'F' => cfg.classify = true, 'i' => cfg.inode = true,
                's' => cfg.size = true, 'r' => cfg.reverse = true,
                'R' => cfg.recursive = true, '1' => cfg.single = true,
                'm' => cfg.comma = true, 'p' => cfg.slash_dir = true,
                'C' => {}
                _ => { eprintln!("ls: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }
    while i < args.len() { paths.push(args[i].clone()); i += 1; }
    if paths.is_empty() { paths.push(".".to_string()); }

    let multi = paths.len() > 1;
    let mut first = true;

    for path_str in &paths {
        let path = Path::new(path_str);
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if cfg.dir || !meta.is_dir() {
                    let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default().to_string();
                    let name_display = if name.is_empty() { path_str.clone() } else { name };
                    if multi && !first { println!(); }
                    if multi || cfg.recursive {
                        if !first { println!(); }
                        println!("{}:", path_str);
                        first = false;
                    }
                    print_entry(&name_display, &meta, Path::new(path_str), &cfg);
                } else {
                    list_dir(path, path_str, &cfg, multi, &mut first);
                }
            }
            Err(e) => { eprintln!("ls: cannot access '{}': {}", path.display(), e); continue; }
        }
    }
}

fn list_dir(path: &Path, display_name: &str, cfg: &Config, multi: bool, first: &mut bool) {
    let entries = match fs::read_dir(path) {
        Ok(r) => {
            let mut v: Vec<_> = r.filter_map(|e| e.ok()).collect();
            v.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            if cfg.reverse { v.reverse(); }
            v
        }
        Err(e) => { eprintln!("ls: cannot open directory '{}': {}", path.display(), e); return; }
    };

    let mut items: Vec<(Vec<u8>, fs::Metadata, bool)> = Vec::new();
    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.as_encoded_bytes();
        if !cfg.all && name_str.first() == Some(&b'.') { continue; }
        if let Ok(meta) = entry.metadata() {
            let is_dir = meta.is_dir();
            items.push((name_str.to_vec(), meta, is_dir));
        }
    }

    let show_header = multi || cfg.recursive;
    if show_header {
        if !*first { println!(); }
        println!("{}:", display_name);
        *first = false;
    }

    if !cfg.long {
        let names: Vec<String> = items.iter().map(|(name_bytes, meta, is_dir)| {
            let name = String::from_utf8_lossy(name_bytes).to_string();
            let mut n = name.clone();
            if cfg.classify {
                let c = if *is_dir { '/' } else if meta.permissions().mode() & 0o111 != 0 { '*' } else { ' ' };
                if c != ' ' { n.push(c); }
            }
            if cfg.slash_dir && *is_dir { n.push('/'); }
            n
        }).collect();

        if cfg.single {
            for n in &names { println!("{}", n); }
        } else if cfg.comma {
            println!("{}", names.join(", "));
        } else {
            let max_width = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
            let term_width = 80;
            let cols = std::cmp::max(1, term_width / max_width.max(1));
            for (i, n) in names.iter().enumerate() {
                print!("{:<width$}", n, width = max_width);
                if (i + 1) % cols == 0 { println!(); }
            }
            if names.len() % cols != 0 { println!(); }
        }
    } else {
        let total: u64 = items.iter().map(|(_, m, _)| m.blocks()).sum();
        println!("total {}", total);
        for (name_bytes, meta, _) in &items {
            let name = String::from_utf8_lossy(name_bytes);
            println!("{}", list_long(path.join(name.as_ref()).as_path(), &name, meta));
        }
    }

    if cfg.recursive {
        for (name_bytes, _, is_dir) in &items {
            if *is_dir {
                let name = String::from_utf8_lossy(name_bytes).to_string();
                if name == "." || name == ".." { continue; }
                let sub_path = path.join(&name);
                list_dir(&sub_path, sub_path.to_string_lossy().as_ref(), cfg, false, first);
            }
        }
    }
}

fn print_entry(name: &str, meta: &fs::Metadata, full_path: &Path, cfg: &Config) {
    if cfg.long {
        println!("{}", list_long(full_path, name, meta));
    } else {
        let mut display = name.to_string();
        if cfg.classify {
            let c = if meta.is_dir() { '/' } else if meta.permissions().mode() & 0o111 != 0 { '*' } else { ' ' };
            if c != ' ' { display.push(c); }
        }
        println!("{}", display);
    }
}
