fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: localedef [-i inputfile] locale");
        std::process::exit(1);
    }
    let mut i = 1;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-i" => { i += 2; }
            "-f" => { i += 2; }
            "-c" => { i += 1; }
            _ => { break; }
        }
    }
    if i >= args.len() {
        eprintln!("localedef: missing locale name");
        std::process::exit(1);
    }
    let locale = &args[i];
    // On POSIX systems, localedef would compile locale definitions into
    // the locale database. On macOS without a standard locale database,
    // simulate by creating the output directory.
    if locale.contains('/') {
        let _ = std::fs::create_dir_all(locale);
    }
}
