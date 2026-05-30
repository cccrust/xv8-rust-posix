#![allow(unused_assignments)]

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 { Some(args[1].as_str()) } else { None };
    let mut buf: Vec<String> = Vec::new();
    let mut cur = 0usize;
    let mut last_cmd: char = loop { break 'p'; };
    let stdin = io::stdin();
    let _stdout = io::stdout();

    if let Some(f) = filename {
        if let Ok(content) = std::fs::read_to_string(f) {
            buf = content.lines().map(|l| l.to_string()).collect();
            cur = buf.len();
        }
    }

    eprint!(": ");
    io::stderr().flush().ok();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        if line.is_empty() {
            match last_cmd {
                'p' => {
                    if cur > 0 && cur <= buf.len() {
                        println!("{}", buf[cur - 1]);
                    }
                }
                'n' => {
                    if cur > 0 && cur <= buf.len() {
                        println!("{}\t{}", cur, buf[cur - 1]);
                    }
                }
                _ => {}
            }
        }
        let cmd = line.trim();
        if cmd.is_empty() { continue; }
        let ch = cmd.chars().next().unwrap();
        let rest = cmd[1..].trim().to_string();
        match ch {
            'q' | 'Q' => { break; }
            'w' => {
                let fname = if rest.is_empty() { filename.unwrap_or("ed.txt") } else { &rest };
                if let Ok(mut f) = std::fs::File::create(fname) {
                    for l in &buf {
                        writeln!(f, "{}", l).ok();
                    }
                    eprintln!("{}", buf.len());
                } else {
                    eprintln!("?");
                }
            }
            'p' | 'n' | 'l' => {
                last_cmd = ch;
                let range = parse_range(&rest, buf.len());
                for i in range {
                    if i > 0 && i <= buf.len() {
                        if ch == 'n' {
                            println!("{}\t{}", i, buf[i - 1]);
                        } else {
                            println!("{}", buf[i - 1]);
                        }
                    }
                }
            }
            'i' | 'a' | 'c' => {
                let pos = if rest.is_empty() { cur } else { rest.parse::<usize>().unwrap_or(cur) };
                let new_lines: Vec<String> = stdin.lock().lines()
                    .take_while(|l| l.as_ref().map_or(false, |s| s != "."))
                    .filter_map(|l| l.ok())
                    .collect();
                match ch {
                    'i' => {
                        for (j, nl) in new_lines.iter().enumerate() {
                            buf.insert(pos + j, nl.clone());
                        }
                        cur = pos + new_lines.len();
                    }
                    'a' => {
                        let pos = if pos >= buf.len() { buf.len() } else { pos + 1 };
                        for (j, nl) in new_lines.iter().enumerate() {
                            if pos + j <= buf.len() {
                                buf.insert(pos + j, nl.clone());
                            } else {
                                buf.push(nl.clone());
                            }
                        }
                        cur = pos + new_lines.len();
                    }
                    'c' => {
                        let start = if pos > 0 { pos - 1 } else { 0 };
                        let end = if let Some(n) = rest.parse::<usize>().ok() { n } else { pos };
                        let count = end.saturating_sub(start).max(1);
                        for _ in 0..count {
                            if start < buf.len() {
                                buf.remove(start);
                            }
                        }
                        for (j, nl) in new_lines.iter().enumerate() {
                            buf.insert(start + j, nl.clone());
                        }
                        cur = start + new_lines.len();
                    }
                    _ => {}
                }
            }
            'd' => {
                let range = parse_range(&rest, buf.len());
                let mut removed: Vec<usize> = range.collect();
                removed.sort_by(|a, b| b.cmp(a));
                for i in removed {
                    if i > 0 && i <= buf.len() {
                        buf.remove(i - 1);
                    }
                }
                cur = if buf.is_empty() { 0 } else { 1 };
            }
            's' => {
                let parts: Vec<&str> = rest.split('/').collect();
                if parts.len() >= 3 {
                    let old = parts[0];
                    let new = parts[1];
                    for i in 0..buf.len() {
                        if buf[i].contains(old) {
                            buf[i] = buf[i].replace(old, new);
                            cur = i + 1;
                            break;
                        }
                    }
                }
            }
            '=' => {
                println!("{}", buf.len());
            }
            '.' => {}
            ',' | '%' => {
                for i in 1..=buf.len() {
                    println!("{}", buf[i - 1]);
                }
            }
            '+' => {
                let n: usize = rest.parse().unwrap_or(1);
                cur = (cur + n).min(buf.len());
            }
            '-' => {
                let n: usize = rest.parse().unwrap_or(1);
                cur = cur.saturating_sub(n).max(1);
            }
            _ => {
                if let Ok(n) = cmd.parse::<usize>() {
                    if n > 0 && n <= buf.len() {
                        cur = n;
                        println!("{}", buf[n - 1]);
                    } else {
                        eprintln!("?");
                    }
                } else {
                    eprintln!("?");
                }
            }
        }
        last_cmd = ch;
        eprint!(": ");
        io::stderr().flush().ok();
    }
}

fn parse_range(s: &str, max: usize) -> Box<dyn Iterator<Item = usize>> {
    if s.is_empty() {
        return Box::new(1..=max);
    }
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        let start: usize = parts[0].parse().unwrap_or(1);
        let end: usize = parts[1].parse().unwrap_or(max);
        let start = start.max(1).min(max);
        let end = end.max(start).min(max);
        Box::new(start..=end)
    } else {
        if let Ok(n) = s.parse::<usize>() {
            let n = n.max(1).min(max);
            Box::new(n..=n)
        } else {
            Box::new(1..=max)
        }
    }
}
