use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<&str> = Vec::new();
    let mut prog: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-f" && i + 1 < args.len() {
            if let Ok(code) = std::fs::read_to_string(&args[i + 1]) {
                prog = Some(code);
            }
            i += 2;
        } else if !args[i].starts_with('-') {
            if prog.is_none() {
                prog = Some(args[i].clone());
            } else {
                files.push(&args[i]);
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    let prog = prog.unwrap_or_else(|| "{ print }".to_string());

    let pairs = parse_program(&prog);

    for (pat, act) in &pairs {
        if pat == "BEGIN" {
            exec_action(act, "", 0, 0, &[]);
        }
    }

    if files.is_empty() { files.push("-"); }
    for f in files {
        let reader: Box<dyn BufRead> = if f == "-" {
            Box::new(io::stdin().lock())
        } else {
            match std::fs::File::open(f) {
                Ok(file) => Box::new(io::BufReader::new(file)),
                Err(e) => { eprintln!("awk: {}: {}", f, e); continue; }
            }
        };
        let mut nr = 0u32;
        for line in reader.lines() {
            let line = line.unwrap_or_default();
            nr += 1;
            let nf = line.split_whitespace().count();
            let fields: Vec<&str> = line.split_whitespace().collect();
            for (pat, act) in &pairs {
                if pat == "BEGIN" || pat == "END" { continue; }
                if pat.is_empty() || eval_pattern(pat, &line, nr, nf, &fields) {
                    exec_action(act, &line, nr, nf, &fields);
                }
            }
        }
    }

    for (pat, act) in &pairs {
        if pat == "END" {
            exec_action(act, "", 0, 0, &[]);
        }
    }
}

fn parse_program(s: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut remaining = s;
    while !remaining.trim().is_empty() {
        remaining = remaining.trim();
        if remaining.starts_with("BEGIN") || remaining.starts_with("END") {
            let kw_len = if remaining.starts_with("BEGIN") { 5 } else { 3 };
            let pat = remaining[..kw_len].to_string();
            remaining = remaining[kw_len..].trim();
            if remaining.starts_with('{') {
                if let Some(end) = find_matching_brace(remaining) {
                    let act = remaining[1..end].trim().to_string();
                    pairs.push((pat, act));
                    remaining = remaining[end+1..].trim();
                } else { break; }
            } else { break; }
        } else if remaining.starts_with('/') {
            if let Some(slash_end) = remaining[1..].find('/') {
                let pat = remaining[..slash_end+2].to_string();
                remaining = remaining[slash_end+2..].trim();
                if remaining.starts_with('{') {
                    if let Some(end) = find_matching_brace(remaining) {
                        let act = remaining[1..end].trim().to_string();
                        pairs.push((pat, act));
                        remaining = remaining[end+1..].trim();
                    } else { break; }
                } else { break; }
            } else { break; }
        } else if remaining.starts_with('{') {
            if let Some(end) = find_matching_brace(remaining) {
                let act = remaining[1..end].trim().to_string();
                pairs.push((String::new(), act));
                remaining = remaining[end+1..].trim();
            } else { break; }
        } else {
            break;
        }
    }
    pairs
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => { depth -= 1; if depth == 0 { return Some(i); } }
            _ => {}
        }
    }
    None
}

fn eval_pattern(pat: &str, line: &str, _nr: u32, nf: usize, fields: &[&str]) -> bool {
    if pat.is_empty() { return true; }
    if pat == "BEGIN" || pat == "END" { return false; }
    if pat.starts_with('/') && pat.ends_with('/') {
        let re = &pat[1..pat.len()-1];
        return line.contains(re);
    }
    if let Ok(n) = pat.parse::<usize>() {
        return nf >= n && fields.get(n-1).map_or(false, |f| !f.is_empty());
    }
    true
}

fn exec_action(act: &str, line: &str, _nr: u32, _nf: usize, fields: &[&str]) {
    for stmt in act.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() { continue; }
        if stmt == "print" {
            println!("{}", line);
        } else if let Some(rest) = stmt.strip_prefix("print ") {
            let args = rest.trim().split(',').map(|s| s.trim()).collect::<Vec<_>>();
            let parts: Vec<String> = args.iter().map(|a| {
                if *a == "$0" { line.to_string() }
                else if a.starts_with('$') {
                    let n: usize = a[1..].parse().unwrap_or(0);
                    if n > 0 && n <= fields.len() { fields[n-1].to_string() }
                    else { String::new() }
                }
                else if a.starts_with('"') && a.ends_with('"') {
                    a[1..a.len()-1].to_string()
                }
                else { a.to_string() }
            }).collect();
            println!("{}", parts.join(" "));
        }
    }
}
