fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tabs <n>...");
        std::process::exit(1);
    }
    let tabs: Vec<usize> = args[1..].iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    if tabs.is_empty() {
        return;
    }
    // Set terminal tab stops via terminfo or hardcoded escape sequence
    let mut cmd = String::new();
    let mut prev = 0usize;
    for t in &tabs {
        if *t > prev {
            cmd.push_str(&format!("\x1b[{}C", t - prev));
        }
        cmd.push_str("\x1bH");
        prev = *t;
    }
    print!("{}", cmd);
}
