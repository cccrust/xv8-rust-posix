fn escape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('0') => out.push('\0'),
                Some('a') => out.push('\x07'),
                Some('b') => out.push('\x08'),
                Some('v') => out.push('\x0b'),
                Some('f') => out.push('\x0c'),
                Some('c') => break,
                Some(c) => { out.push('\\'); out.push(c); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut no_newline = false;
    let mut interpret_escapes = false;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'n' => no_newline = true,
                'e' => interpret_escapes = true,
                'E' => interpret_escapes = false,
                _ => { break; }
            }
        }
        // GNU echo stops at first non-option arg
        if args[i].starts_with('-') && args[i].len() > 1 {
            let rest: String = args[i][1..].chars().filter(|&c| c == 'n' || c == 'e' || c == 'E').collect();
            if rest.len() < args[i].len() - 1 { break; }
        }
        i += 1;
    }

    let output = if interpret_escapes {
        args[i..].iter().map(|s| escape(s)).collect::<Vec<_>>().join(" ")
    } else {
        args[i..].join(" ")
    };
    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}
