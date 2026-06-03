#![allow(unused_assignments)]

use std::io::{self, BufRead, Write};

fn read_lines() -> Vec<String> {
    io::stdin().lock().lines()
        .take_while(|l| l.as_ref().map_or(false, |s| s != "."))
        .filter_map(|l| l.ok())
        .collect()
}

fn parse_range(s: &str, max: usize) -> (usize, usize, char) {
    let s = s.trim();
    if s.is_empty() { return (1, max, 'p'); }
    let ch = s.chars().next().unwrap();
    if !ch.is_digit(10) && ch != '.' && ch != '$' && ch != ',' && ch != ';' && ch != '%' && ch != '\'' && ch != '/' && ch != '?' {
        return (1, max, ch);
    }
    let cmd_start = {
        let mut i = s.len();
        for (j, c) in s.char_indices() {
            if c.is_alphabetic() || c == '=' || c == '!' || c == '"' {
                i = j;
                break;
            }
        }
        i
    };
    let addr_str = s[..cmd_start].trim();
    let cmd = if cmd_start < s.len() { s[cmd_start..].chars().next().unwrap_or('p') } else { 'p' };
    let rest = if cmd_start + 1 < s.len() { s[cmd_start+1..].trim().to_string() } else { String::new() };

    if addr_str == "%" || addr_str == "1,$" { return (1, max, cmd); }
    if addr_str.is_empty() || addr_str == "." { return (1, max, cmd); }

    let parts: Vec<&str> = addr_str.split(',').collect();
    if parts.len() == 2 {
        let start = parse_addr(parts[0], max);
        let end = parse_addr(parts[1], max);
        return (start, end, cmd);
    }
    let parts2: Vec<&str> = addr_str.split(';').collect();
    if parts2.len() == 2 {
        let start = parse_addr(parts2[0], max);
        let end = parse_addr(parts2[1], max);
        return (start.max(end), end, cmd);
    }
    let n = parse_addr(addr_str, max);
    (n, n, cmd)
}

fn parse_addr(s: &str, max: usize) -> usize {
    let s = s.trim();
    if s == "." || s.is_empty() { return 1; }
    if s == "$" { return max; }
    s.parse().unwrap_or(1).clamp(1, max)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 { Some(args[1].as_str()) } else { None };
    let mut buf: Vec<String> = Vec::new();
    let mut cur = 0usize;
    let mut last_cmd: char = 'p';
    let stdin = io::stdin();

    if let Some(f) = filename {
        if let Ok(content) = std::fs::read_to_string(f) {
            buf = content.lines().map(|l| l.to_string()).collect();
            cur = buf.len();
            eprintln!("{}", buf.len());
        }
    }

    eprint!(":");
    io::stderr().flush().ok();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        if line.trim().is_empty() {
            if last_cmd == 'p' || last_cmd == 'n' || last_cmd == 'l' {
                if cur > 0 && cur <= buf.len() {
                    if last_cmd == 'n' {
                        println!("{}\t{}", cur, buf[cur - 1]);
                    } else {
                        println!("{}", buf[cur - 1]);
                    }
                }
            }
            eprint!(":");
            io::stderr().flush().ok();
            continue;
        }

        let input = line.trim().to_string();
        let (start, end, cmd) = parse_range(&input, if buf.is_empty() { 1 } else { buf.len() });
        let rest = {
            let mut i = 0;
            for (j, c) in input.char_indices() {
                if c.is_alphabetic() || c == '=' || c == '!' || c == '"' {
                    i = j;
                    break;
                }
            }
            if i > 0 && i < input.len() { input[i+1..].trim().to_string() } else { String::new() }
        };

        match cmd {
            'q' => { break; }
            'Q' => { break; }
            'x' | 'w' if cmd == 'x' || (cmd == 'w' && rest == "q") => {
                let fname = if rest.is_empty() { filename.unwrap_or("ex.txt") } else { &rest };
                if let Ok(mut f) = std::fs::File::create(fname) {
                    for l in &buf { writeln!(f, "{}", l).ok(); }
                    eprintln!("{}", buf.len());
                }
                if cmd == 'x' || rest == "q" { break; }
            }
            'w' => {
                let fname = if rest.is_empty() { filename.unwrap_or("ex.txt") } else { &rest };
                if let Ok(mut f) = std::fs::File::create(fname) {
                    for l in &buf { writeln!(f, "{}", l).ok(); }
                    eprintln!("{}", buf.len());
                } else { eprintln!("?"); }
            }
            'e' => {
                let fname = if rest.is_empty() { filename.unwrap_or("ex.txt") } else { &rest };
                if let Ok(content) = std::fs::read_to_string(fname) {
                    buf = content.lines().map(|l| l.to_string()).collect();
                    cur = buf.len();
                    eprintln!("{}", buf.len());
                } else { eprintln!("?"); }
            }
            'p' | 'n' | 'l' | '#' | 'P' => {
                last_cmd = if cmd == '#' { 'n' } else if cmd == 'P' { 'p' } else { cmd };
                for i in std::cmp::max(1, start)..=std::cmp::min(end, buf.len()) {
                    if i > 0 && i <= buf.len() {
                        if cmd == 'n' || cmd == '#' {
                            println!("{}\t{}", i, buf[i - 1]);
                        } else {
                            println!("{}", buf[i - 1]);
                        }
                    }
                }
            }
            'z' => {
                let n: usize = rest.parse().unwrap_or(20);
                for i in start..std::cmp::min(start + n, buf.len() + 1) {
                    if i <= buf.len() { println!("{}", buf[i - 1]); }
                }
            }
            'i' | 'a' | 'c' => {
                if cmd == 'c' && rest.starts_with('o') {
                    // co (copy) = t
                    let dest: usize = rest.trim_start_matches('o').trim().parse().unwrap_or(1);
                    let dest = dest.max(1).min(buf.len().max(1));
                    let mut extracted: Vec<String> = (start..=end)
                        .filter(|&i| i > 0 && i <= buf.len())
                        .map(|i| buf[i - 1].clone())
                        .collect();
                    let indices: Vec<usize> = (start..=end).collect();
                    for idx in indices.iter().rev() {
                        if *idx > 0 && *idx <= buf.len() { buf.remove(*idx - 1); }
                    }
                    let insert_at = if dest > start { dest.saturating_sub(extracted.len()) } else { dest };
                    for (j, line) in extracted.iter().enumerate() {
                        let pos = insert_at + j;
                        if pos <= buf.len() { buf.insert(pos, line.clone()); }
                        else { buf.push(line.clone()); }
                    }
                    eprint!(":");
                    io::stderr().flush().ok();
                    continue;
                }
                let new_lines = read_lines();
                match cmd {
                    'i' => {
                        for (j, nl) in new_lines.iter().enumerate() {
                            buf.insert(start + j, nl.clone());
                        }
                        cur = start + new_lines.len();
                    }
                    'a' => {
                        let pos = if start >= buf.len() { buf.len() } else { start + 1 };
                        for (j, nl) in new_lines.iter().enumerate() {
                            if pos + j <= buf.len() { buf.insert(pos + j, nl.clone()); }
                            else { buf.push(nl.clone()); }
                        }
                        cur = pos + new_lines.len();
                    }
                    'c' => {
                        let count = end.saturating_sub(start).max(1);
                        for _ in 0..count {
                            if start <= buf.len() { buf.remove(start - 1); }
                        }
                        for (j, nl) in new_lines.iter().enumerate() {
                            buf.insert(start - 1 + j, nl.clone());
                        }
                        cur = start - 1 + new_lines.len();
                    }
                    _ => {}
                }
            }
            'd' => {
                let mut indices: Vec<usize> = (start..=end).collect();
                indices.sort_by(|a, b| b.cmp(a));
                for i in indices {
                    if i > 0 && i <= buf.len() { buf.remove(i - 1); }
                }
                cur = if buf.is_empty() { 0 } else { 1.min(buf.len()) };
            }
            's' => {
                let delim = rest.chars().next().unwrap_or('/');
                let mut parts = rest.split(delim);
                let old = parts.nth(1).unwrap_or("");
                let new = parts.next().unwrap_or("");
                let flags = parts.next().unwrap_or("");
                let global = flags.contains('g');
                let count = if global { usize::MAX } else { 1 };
                let mut replaced = 0usize;
                for i in start..=end {
                    if i > 0 && i <= buf.len() && replaced < count {
                        if buf[i - 1].contains(old) {
                            buf[i - 1] = buf[i - 1].replacen(old, new, 1);
                            cur = i;
                            replaced += 1;
                        }
                    }
                }
                if cur > 0 && cur <= buf.len() {
                    println!("{}", buf[cur - 1]);
                }
            }
            'g' | 'v' => {
                let delim = rest.chars().next().unwrap_or('/');
                let rest2 = rest.trim_start_matches(delim);
                let pattern = rest2.split(delim).next().unwrap_or("");
                let sub_cmd = rest2.split(delim).skip(1).next().unwrap_or("p").trim().chars().next().unwrap_or('p');
                let invert = cmd == 'v';
                for i in start..=std::cmp::min(end, buf.len()) {
                    if i > 0 && i <= buf.len() {
                        let matches = buf[i - 1].contains(pattern);
                        if matches != invert {
                            if sub_cmd == 'p' || sub_cmd == 'l' || sub_cmd == 'n' {
                                if sub_cmd == 'n' { println!("{}\t{}", i, buf[i - 1]); }
                                else { println!("{}", buf[i - 1]); }
                            } else if sub_cmd == 'd' {
                                buf.remove(i - 1);
                            }
                        }
                    }
                }
            }
            'm' | 't' => {
                let dest: usize = rest.parse().unwrap_or(1);
                let dest = dest.max(1).min(buf.len());
                let mut extracted: Vec<String> = (start..=end)
                    .filter(|&i| i > 0 && i <= buf.len())
                    .map(|i| buf[i - 1].clone())
                    .collect();
                let mut indices: Vec<usize> = (start..=end).collect();
                indices.sort_by(|a, b| b.cmp(a));
                for i in &indices {
                    if *i > 0 && *i <= buf.len() { buf.remove(*i - 1); }
                }
                let insert_at = if dest > start { dest.saturating_sub(indices.len()) } else { dest };
                for (j, line) in extracted.iter().enumerate() {
                    let pos = if cmd == 'm' { insert_at + j } else { dest + j + 1 };
                    if pos <= buf.len() { buf.insert(pos, line.clone()); }
                    else { buf.push(line.clone()); }
                }
            }
            'k' => {
                // mark: just sets current line
                cur = start;
            }
            'u' => {
                eprintln!("?");
            }
            '=' => {
                println!("{}", buf.len());
            }
            'f' => {
                if !rest.is_empty() {
                    // set filename
                }
                eprintln!("{}", filename.unwrap_or("[No File]"));
            }
            'r' => {
                let fname = if rest.is_empty() { "ex.txt" } else { &rest };
                if let Ok(content) = std::fs::read_to_string(fname) {
                    for l in content.lines() {
                        buf.push(l.to_string());
                    }
                    eprintln!("{}", buf.len());
                } else { eprintln!("?"); }
            }
            'j' => {
                let joined: String = (start..=std::cmp::min(end, buf.len()))
                    .filter(|&i| i > 0 && i <= buf.len())
                    .map(|i| buf[i - 1].clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                let indices: Vec<usize> = (start..=end).filter(|&i| i > 0 && i <= buf.len()).collect();
                let first = *indices.first().unwrap_or(&1);
                for i in indices.iter().rev() {
                    if *i > 0 && *i <= buf.len() { buf.remove(*i - 1); }
                }
                if first <= buf.len() { buf.insert(first - 1, joined); }
                else { buf.push(joined); }
            }
            '.' => {
                if cur > 0 && cur <= buf.len() {
                    println!("{}", buf[cur - 1]);
                }
            }
            '$' => {
                cur = buf.len();
                if cur > 0 { println!("{}", buf[cur - 1]); }
            }
            _ => {
                if let Ok(n) = input.trim().parse::<usize>() {
                    if n > 0 && n <= buf.len() {
                        cur = n;
                        println!("{}", buf[n - 1]);
                    } else { eprintln!("?"); }
                } else {
                    eprintln!("?");
                }
            }
        }
        if cmd != 'p' && cmd != 'n' && cmd != 'l' && cmd != '#' && input.chars().any(|c| c.is_alphabetic()) {
            // don't update last_cmd for address-only commands
        }
        last_cmd = if cmd == 'P' { 'p' } else if cmd == '#' { 'n' } else { cmd };
        eprint!(":");
        io::stderr().flush().ok();
    }
}
