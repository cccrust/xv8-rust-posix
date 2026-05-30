use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn apply_mode(path: &Path, mode: u32, recursive: bool) {
    if recursive && path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                apply_mode(&entry.path(), mode, true);
            }
        }
    }
    if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(mode)) {
        eprintln!("chmod: cannot change mode of '{}': {}", path.display(), e);
        std::process::exit(1);
    }
}

fn parse_mode(s: &str, current_mode: u32) -> Option<u32> {
    if s.as_bytes().first()?.is_ascii_digit() {
        return u32::from_str_radix(s, 8).ok();
    }
    // Symbolic mode like u+rwx, go-w, a+r etc.
    let mut mode = current_mode;
    let mut remaining = s;
    while !remaining.is_empty() {
        let who_end = remaining.find(|c: char| c == '+' || c == '-' || c == '=').unwrap_or(remaining.len());
        let who = if who_end == 0 { "a" } else { &remaining[..who_end] };
        let op = remaining[who_end..].chars().next()?;
        let perm_start = who_end + 1;
        let perm_end = remaining[perm_start..].find(|c: char| c == ',').map(|i| perm_start + i).unwrap_or(remaining.len());
        let perms = &remaining[perm_start..perm_end];

        let who_mask = if who.contains('u') { 0o7700 } else { 0 }
            | if who.contains('g') { 0o7070 } else { 0 }
            | if who.contains('o') { 0o7007 } else { 0 }
            | if who.is_empty() || who.contains('a') { 0o7777 } else { 0 };

        let perm_bits = if perms.contains('r') { 0o444 } else { 0 }
            | if perms.contains('w') { 0o222 } else { 0 }
            | if perms.contains('x') { 0o111 } else { 0 }
            | if perms.contains('s') { 0o6000 } else { 0 }
            | if perms.contains('t') { 0o1000 } else { 0 };

        match op {
            '+' => mode |= perm_bits & who_mask,
            '-' => mode &= !(perm_bits & who_mask),
            '=' => mode = (mode & !who_mask) | (perm_bits & who_mask),
            _ => return None,
        }

        if perm_end >= remaining.len() { break; }
        remaining = &remaining[perm_end + 1..];
    }
    Some(mode)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut recursive = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'R' => recursive = true,
                _ => { eprintln!("chmod: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i + 1 >= args.len() {
        eprintln!("usage: chmod [-R] mode file ...");
        std::process::exit(1);
    }

    let mode_str = &args[i];
    i += 1;

    for path_str in &args[i..] {
        let path = Path::new(path_str);
        let current = fs::metadata(path).ok().map(|m| m.permissions().mode()).unwrap_or(0);
        let new_mode = match parse_mode(mode_str, current) {
            Some(m) => m,
            None => { eprintln!("chmod: invalid mode: '{}'", mode_str); std::process::exit(1); }
        };
        apply_mode(path, new_mode, recursive);
    }
}
