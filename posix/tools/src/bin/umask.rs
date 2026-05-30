fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        #[cfg(unix)]
        unsafe {
            let mask = libc::umask(0);
            libc::umask(mask);
            println!("{:04o}", mask);
        }
        return;
    }
    let mut i = 1;
    let mut symbolic = false;
    if args[i] == "-S" {
        symbolic = true;
        i += 1;
    }
    if i >= args.len() { return; }
    if symbolic {
        let mask = parse_symbolic(&args[i]);
        #[cfg(unix)]
        unsafe { libc::umask(mask as libc::mode_t); }
    } else {
        if let Ok(mask) = usize::from_str_radix(&args[i], 8) {
            #[cfg(unix)]
            unsafe { libc::umask(mask as libc::mode_t); }
        }
    }
}

fn parse_symbolic(s: &str) -> u32 {
    let mut mask = 0u32;
    for ch in s.chars() {
        match ch {
            'r' => mask |= 4,
            'w' => mask |= 2,
            'x' => mask |= 1,
            'u' | 'g' | 'o' | 'a' | '=' | '+' | '-' => {}
            _ => {}
        }
    }
    mask
}
