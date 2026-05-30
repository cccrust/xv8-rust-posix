use std::fs::File;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        eprintln!("nohup: invalid option -- '{}'", args[i]);
        std::process::exit(1);
    }

    if i >= args.len() {
        eprintln!("usage: nohup command [arg ...]");
        std::process::exit(1);
    }

    let cmd = &args[i];
    let cmd_args: Vec<&str> = args[i + 1..].iter().map(String::as_str).collect();

    // Always redirect stdout to nohup.out
    let f = File::options().append(true).create(true).open("nohup.out")
        .unwrap_or_else(|e| {
            eprintln!("nohup: cannot open nohup.out: {}", e);
            std::process::exit(1);
        });

    let mut child = Command::new(cmd)
        .args(&cmd_args)
        .stdout(f)
        .stdin(std::process::Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("nohup: {}: {}", cmd, e);
            std::process::exit(1);
        });

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("nohup: {}", e);
        std::process::exit(1);
    });

    std::process::exit(status.code().unwrap_or(1));
}
