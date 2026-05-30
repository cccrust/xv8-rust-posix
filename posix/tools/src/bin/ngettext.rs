fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: ngettext msgid msgid_plural n");
        std::process::exit(1);
    }
    let msgid = &args[1];
    let msgid_plural = &args[2];
    let n: u64 = args[3].parse().unwrap_or(0);
    if n == 1 {
        println!("{}", msgid);
    } else {
        println!("{}", msgid_plural);
    }
}
