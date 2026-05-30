use std::io::{self, BufRead};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'I' => {} // replace occurrence
                'n' => {} // max args
                'P' => {} // max processes
                '0' => {} // null input
                _ => { eprintln!("xargs: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let cmd_args: Vec<&str> = args[i..].iter().map(String::as_str).collect();
    if cmd_args.is_empty() {
        eprintln!("usage: xargs command [arg ...]");
        std::process::exit(1);
    }

    let cmd_name = cmd_args[0];
    let base_args: Vec<&str> = cmd_args[1..].to_vec();

    let stdin = io::stdin();
    let mut items: Vec<String> = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default().trim().to_string();
        if line.is_empty() { continue; }
        // Split on whitespace
        for word in line.split_whitespace() {
            items.push(word.to_string());
        }
    }

    if items.is_empty() { return; }

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
