fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: c99 [-o output] file...");
        std::process::exit(1);
    }
    let mut cmd = std::process::Command::new("cc");
    cmd.args(&args[1..]);
    let status = cmd.status().unwrap_or_else(|_| {
        eprintln!("c99: cc not found");
        std::process::exit(127);
    });
    std::process::exit(status.code().unwrap_or(1));
}
