use std::process::Command;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: time <command> [args...]");
        std::process::exit(1);
    }
    let start = Instant::now();
    let child = Command::new(&args[1]).args(&args[2..]).spawn();
    match child {
        Ok(mut c) => {
            let status = c.wait();
            let elapsed = start.elapsed();
            match status {
                Ok(s) => {
                    eprintln!("real\t{}.{:03}s", elapsed.as_secs(), elapsed.subsec_millis());
                    if !s.success() {
                        std::process::exit(s.code().unwrap_or(1));
                    }
                }
                Err(e) => {
                    eprintln!("time: wait: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("time: cannot run {}: {}", args[1], e);
            std::process::exit(127);
        }
    }
}
