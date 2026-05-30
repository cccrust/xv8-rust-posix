use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: command <command> [args...]");
        std::process::exit(1);
    }
    let mut i = 1;
    let mut only_external = false;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-p" => { i += 1; }
            "-v" | "-V" => { only_external = true; i += 1; }
            _ => { break; }
        }
    }
    if i >= args.len() {
        eprintln!("Usage: command <command> [args...]");
        std::process::exit(1);
    }
    let cmd = &args[i];
    let cmd_args: Vec<&str> = args[i+1..].iter().map(|s| s.as_str()).collect();

    if only_external {
        let paths = std::env::var("PATH").unwrap_or_default();
        for dir in paths.split(':') {
            let p = format!("{}/{}", dir, cmd);
            if std::path::Path::new(&p).is_file() {
                println!("{}", p);
                return;
            }
        }
        std::process::exit(1);
    }

    let child = Command::new(cmd).args(&cmd_args).spawn();
    match child {
        Ok(mut c) => {
            let status = c.wait();
            match status {
                Ok(s) => { std::process::exit(s.code().unwrap_or(0)); }
                Err(e) => {
                    eprintln!("command: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("command: {}: {}", cmd, e);
            std::process::exit(127);
        }
    }
}
