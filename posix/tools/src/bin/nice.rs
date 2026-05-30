use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut adjustment = 10i32;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        if args[i] == "-n" {
            // -n is used for adjusting niceness value
            i += 1;
            if i < args.len() {
                adjustment = args[i].parse().unwrap_or(10);
            }
        } else {
            eprintln!("nice: invalid option -- '{}'", args[i]);
            std::process::exit(1);
        }
        i += 1;
    }

    if i >= args.len() {
        // Print current niceness
        #[cfg(unix)]
        unsafe {
            println!("{}", libc::getpriority(libc::PRIO_PROCESS, 0));
        }
        #[cfg(not(unix))]
        println!("0");
        return;
    }

    let cmd = &args[i];
    let cmd_args: Vec<&str> = args[i + 1..].iter().map(String::as_str).collect();

    #[cfg(unix)]
    {
        // On Unix, nice() sets the nice value then exec
        unsafe { libc::nice(adjustment); }
    }

    let mut child = Command::new(cmd);
    child.args(&cmd_args);

    let status = child.status().unwrap_or_else(|e| {
        eprintln!("nice: {}: {}", cmd, e);
        std::process::exit(1);
    });

    std::process::exit(status.code().unwrap_or(1));
}
