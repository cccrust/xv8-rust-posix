fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: printf <format> [arguments...]");
        std::process::exit(1);
    }
    let fmt = &args[1];
    let mut i = 0;
    let mut arg_idx = 2;
    let fmt_chars: Vec<char> = fmt.chars().collect();
    while i < fmt_chars.len() {
        if fmt_chars[i] == '\\' && i + 1 < fmt_chars.len() {
            match fmt_chars[i + 1] {
                '"' => print!("\""),
                '\\' => print!("\\"),
                'a' => print!("\x07"),
                'b' => print!("\x08"),
                'c' => { return; }
                'f' => print!("\x0C"),
                'n' => print!("\n"),
                'r' => print!("\r"),
                't' => print!("\t"),
                'v' => print!("\x0B"),
                '0'..='7' => {
                    let mut oct = String::new();
                    let mut j = i + 1;
                    while j < fmt_chars.len() && fmt_chars[j] >= '0' && fmt_chars[j] <= '7' && oct.len() < 3 {
                        oct.push(fmt_chars[j]);
                        j += 1;
                    }
                    if let Ok(val) = u32::from_str_radix(&oct, 8) {
                        print!("{}", std::char::from_u32(val).unwrap_or('\x00'));
                    }
                    i = j - 1;
                }
                'x' => {
                    let mut hex = String::new();
                    let mut j = i + 2;
                    while j < fmt_chars.len() && fmt_chars[j].is_ascii_hexdigit() && hex.len() < 2 {
                        hex.push(fmt_chars[j]);
                        j += 1;
                    }
                    if !hex.is_empty() {
                        if let Ok(val) = u32::from_str_radix(&hex, 16) {
                            print!("{}", std::char::from_u32(val).unwrap_or('\x00'));
                        }
                        i = j - 1;
                    } else {
                        print!("x");
                    }
                }
                _ => print!("{}", fmt_chars[i + 1]),
            }
            i += 2;
        } else if fmt_chars[i] == '%' && i + 1 < fmt_chars.len() {
            i += 1;
            let mut flags = String::new();
            while i < fmt_chars.len() && "-+ #0'".contains(fmt_chars[i]) {
                flags.push(fmt_chars[i]);
                i += 1;
            }
            let mut width_str = String::new();
            if i < fmt_chars.len() && fmt_chars[i] == '*' {
                if arg_idx < args.len() {
                    width_str = args[arg_idx].clone();
                    arg_idx += 1;
                }
                i += 1;
            } else {
                while i < fmt_chars.len() && fmt_chars[i].is_ascii_digit() {
                    width_str.push(fmt_chars[i]);
                    i += 1;
                }
            }
            let mut prec_str = String::new();
            if i < fmt_chars.len() && fmt_chars[i] == '.' {
                i += 1;
                if i < fmt_chars.len() && fmt_chars[i] == '*' {
                    if arg_idx < args.len() {
                        prec_str = args[arg_idx].clone();
                        arg_idx += 1;
                    }
                    i += 1;
                } else {
                    while i < fmt_chars.len() && fmt_chars[i].is_ascii_digit() {
                        prec_str.push(fmt_chars[i]);
                        i += 1;
                    }
                }
            }
            if i >= fmt_chars.len() { break; }
            let spec = fmt_chars[i];
            let arg = if arg_idx < args.len() {
                args[arg_idx].clone()
            } else {
                String::new()
            };
            let width: usize = width_str.parse().unwrap_or(0);
            let prec: isize = prec_str.parse().unwrap_or(-1);
            match spec {
                'd' | 'i' => {
                    let val: i64 = arg.parse().unwrap_or(0);
                    let s = format!("{}", val);
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", s, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", s, width = width);
                    } else {
                        print!("{}", s);
                    }
                }
                'u' => {
                    let val: u64 = arg.parse().unwrap_or(0);
                    let s = format!("{}", val);
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", s, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", s, width = width);
                    } else {
                        print!("{}", s);
                    }
                }
                'o' => {
                    let val: u64 = arg.parse().unwrap_or(0);
                    let s = format!("{:o}", val);
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", s, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", s, width = width);
                    } else {
                        print!("{}", s);
                    }
                }
                'x' => {
                    let val: u64 = arg.parse().unwrap_or(0);
                    let s = format!("{:x}", val);
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", s, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", s, width = width);
                    } else {
                        print!("{}", s);
                    }
                }
                'X' => {
                    let val: u64 = arg.parse().unwrap_or(0);
                    let s = format!("{:X}", val);
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", s, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", s, width = width);
                    } else {
                        print!("{}", s);
                    }
                }
                'f' | 'F' => {
                    let val: f64 = arg.parse().unwrap_or(0.0);
                    if prec >= 0 {
                        print!("{:.prec$}", val, prec = prec as usize);
                    } else {
                        print!("{}", val);
                    }
                }
                'e' => {
                    let val: f64 = arg.parse().unwrap_or(0.0);
                    if prec >= 0 {
                        print!("{:.prec$e}", val, prec = prec as usize);
                    } else {
                        print!("{:e}", val);
                    }
                }
                'E' => {
                    let val: f64 = arg.parse().unwrap_or(0.0);
                    if prec >= 0 {
                        let s = format!("{:.prec$e}", val, prec = prec as usize);
                        print!("{}", s.to_uppercase());
                    } else {
                        let s = format!("{:e}", val);
                        print!("{}", s.to_uppercase());
                    }
                }
                'g' => {
                    let val: f64 = arg.parse().unwrap_or(0.0);
                    if prec >= 0 {
                        let s = format!("{:.prec$}", val, prec = prec as usize);
                        print!("{}", s);
                    } else {
                        print!("{}", val);
                    }
                }
                'G' => {
                    let val: f64 = arg.parse().unwrap_or(0.0);
                    if prec >= 0 {
                        let s = format!("{:.prec$}", val, prec = prec as usize);
                        print!("{}", s.to_uppercase());
                    } else {
                        let s = format!("{}", val).to_uppercase();
                        print!("{}", s);
                    }
                }
                's' => {
                    if width > 0 && !flags.contains('-') {
                        print!("{:>width$}", arg, width = width);
                    } else if width > 0 {
                        print!("{:<width$}", arg, width = width);
                    } else {
                        print!("{}", arg);
                    }
                }
                'c' => {
                    let c = arg.chars().next().unwrap_or('\x00');
                    print!("{}", c);
                }
                '%' => print!("%"),
                'b' => {
                    let mut j = 0;
                    let bchars: Vec<char> = arg.chars().collect();
                    while j < bchars.len() {
                        if bchars[j] == '\\' && j + 1 < bchars.len() {
                            match bchars[j + 1] {
                                '"' => print!("\""),
                                '\\' => print!("\\"),
                                'a' => print!("\x07"),
                                'b' => print!("\x08"),
                                'c' => { break; }
                                'f' => print!("\x0C"),
                                'n' => print!("\n"),
                                'r' => print!("\r"),
                                't' => print!("\t"),
                                'v' => print!("\x0B"),
                                '0'..='7' => {
                                    let mut oct = String::new();
                                    let mut k = j + 1;
                                    while k < bchars.len() && bchars[k] >= '0' && bchars[k] <= '7' && oct.len() < 3 {
                                        oct.push(bchars[k]);
                                        k += 1;
                                    }
                                    if let Ok(val) = u32::from_str_radix(&oct, 8) {
                                        print!("{}", std::char::from_u32(val).unwrap_or('\x00'));
                                    }
                                    j = k - 1;
                                }
                                'x' => {
                                    let mut hex = String::new();
                                    let mut k = j + 2;
                                    while k < bchars.len() && bchars[k].is_ascii_hexdigit() && hex.len() < 2 {
                                        hex.push(bchars[k]);
                                        k += 1;
                                    }
                                    if !hex.is_empty() {
                                        if let Ok(val) = u32::from_str_radix(&hex, 16) {
                                            print!("{}", std::char::from_u32(val).unwrap_or('\x00'));
                                        }
                                        j = k - 1;
                                    } else {
                                        print!("x");
                                    }
                                }
                                _ => print!("{}", bchars[j + 1]),
                            }
                            j += 2;
                        } else {
                            print!("{}", bchars[j]);
                            j += 1;
                        }
                    }
                }
                _ => {
                    if !arg.is_empty() {
                        arg_idx += 1;
                    }
                }
            }
            if spec != 'b' && !arg.is_empty() {
                arg_idx += 1;
            }
            i += 1;
        } else {
            print!("{}", fmt_chars[i]);
            i += 1;
        }
    }
}
