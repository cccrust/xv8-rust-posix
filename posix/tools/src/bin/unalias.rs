fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: unalias <name>...");
        std::process::exit(1);
    }
    let mut i = 1;
    let mut all = false;
    if args[i] == "-a" {
        all = true;
        i += 1;
    }
    if all {
        // no persistent alias storage, so nothing to do
        return;
    }
    for _ in &args[i..] {
        // no persistent alias storage, so nothing to remove
    }
}
