fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        // Print default aliases
        println!("ls='ls --color=auto'");
        println!("grep='grep --color=auto'");
        println!("..='cd ..'");
        return;
    }
    for arg in &args[1..] {
        if arg.contains('=') {
            // Setting alias - no persistent storage between invocations
            // Just silently accept
        } else {
            eprintln!("alias: {}: not found", arg);
            std::process::exit(1);
        }
    }
}
