fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tput <capname> [args...]");
        std::process::exit(1);
    }
    let cap = &args[1];
    match cap.as_str() {
        "clear" => print!("\x1b[2J\x1b[H"),
        "cols" | "columns" => {
            if let Ok(size) = term_size() {
                println!("{}", size.0);
            } else {
                println!("80");
            }
        }
        "lines" | "rows" => {
            if let Ok(size) = term_size() {
                println!("{}", size.1);
            } else {
                println!("24");
            }
        }
        "bold" => print!("\x1b[1m"),
        "smso" | "rev" => print!("\x1b[7m"),
        "rmso" => print!("\x1b[27m"),
        "sgr0" => print!("\x1b[0m"),
        "setaf" | "AF" => {
            if args.len() > 2 {
                let color: u8 = args[2].parse().unwrap_or(7);
                print!("\x1b[38;5;{}m", color);
            }
        }
        "setab" | "AB" => {
            if args.len() > 2 {
                let color: u8 = args[2].parse().unwrap_or(0);
                print!("\x1b[48;5;{}m", color);
            }
        }
        "cup" => {
            if args.len() > 3 {
                let row: u16 = args[2].parse().unwrap_or(0);
                let col: u16 = args[3].parse().unwrap_or(0);
                print!("\x1b[{};{}H", row + 1, col + 1);
            }
        }
        "bel" => print!("\x07"),
        "civis" => print!("\x1b[?25l"),
        "cnorm" => print!("\x1b[?25h"),
        "el" => print!("\x1b[K"),
        "ed" => print!("\x1b[J"),
        "home" => print!("\x1b[H"),
        "init" | "reset" => print!("\x1bc"),
        _ => {
            eprintln!("tput: unknown capability: {}", cap);
            std::process::exit(1);
        }
    }
}

fn term_size() -> Result<(u16, u16), ()> {
    #[cfg(unix)]
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws as *mut _ as *mut _) == 0 {
            return Ok((ws.ws_col, ws.ws_row));
        }
    }
    Err(())
}
