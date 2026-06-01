use std::io::{self, BufRead};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut max_args: usize = 0;
    let mut replace_str: Option<String> = None;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        let arg = args[i].clone();
        let opt_chars: Vec<char> = arg[1..].chars().collect();
        let mut j = 0;
        while j < opt_chars.len() {
            match opt_chars[j] {
                'I' => {
                    if j + 1 < opt_chars.len() {
                        replace_str = Some(opt_chars[j + 1..].iter().collect());
                        j = opt_chars.len();
                    } else {
                        i += 1;
                        if i < args.len() { replace_str = Some(args[i].clone()); }
                    }
                }
                'n' => {
                    if j + 1 < opt_chars.len() {
                        let val: String = opt_chars[j + 1..].iter().collect();
                        max_args = val.parse().unwrap_or(0);
                        j = opt_chars.len();
                    } else {
                        i += 1;
                        if i < args.len() { max_args = args[i].parse().unwrap_or(0); }
                    }
                }
                'P' => { /* max processes - ignore */ }
                '0' => { /* null input - ignore */ }
                _ => { eprintln!("xargs: invalid option -- '{}'", opt_chars[j]); std::process::exit(1); }
            }
            j += 1;
        }
        i += 1;
    }

    let cmd_args: Vec<&str> = args[i..].iter().map(String::as_str).collect();
    let (cmd_name, base_args): (&str, Vec<&str>) = if cmd_args.is_empty() {
        ("echo", vec![])
    } else {
        (cmd_args[0], cmd_args[1..].to_vec())
    };

    let stdin = io::stdin();
    let mut items: Vec<String> = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() { continue; }
        for word in trimmed.split_whitespace() {
            items.push(word.to_string());
        }
    }

    if items.is_empty() { return; }

    if let Some(ref repl) = replace_str {
        for item in &items {
            let mut child_args: Vec<String> = Vec::new();
            for a in &base_args {
                child_args.push(a.replace(repl, item));
            }
            if child_args.is_empty() {
                child_args.push(item.clone());
            }
            let mut child = Command::new(cmd_name);
            child.args(&child_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let status = child.status().unwrap_or_else(|e| {
                eprintln!("xargs: {}: {}", cmd_name, e);
                std::process::exit(1);
            });

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    } else if max_args > 0 {
        for chunk in items.chunks(max_args) {
            let mut child = Command::new(cmd_name);
            child.args(&base_args).args(chunk)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let status = child.status().unwrap_or_else(|e| {
                eprintln!("xargs: {}: {}", cmd_name, e);
                std::process::exit(1);
            });

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    } else {
        for item in &items {
            let mut child = Command::new(cmd_name);
            child.args(&base_args).arg(item)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let status = child.status().unwrap_or_else(|e| {
                eprintln!("xargs: {}: {}", cmd_name, e);
                std::process::exit(1);
            });

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
}
