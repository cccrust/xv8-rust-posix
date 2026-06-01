use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <user> <host> <command>", args[0]);
        eprintln!("Example: {} user@example.com ls", args[0]);
        exit(1);
    }

    // Placeholder: just print the arguments and exit
    println!("ssh_client: connecting to {}@{}: {}", args[1], args[2], &args[3..].join(" "));
    println!("ssh_client: This is a placeholder implementation.");
    exit(0);
}