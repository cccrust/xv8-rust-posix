fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-a" {
        println!("C");
        println!("POSIX");
        println!("en_US.UTF-8");
        return;
    }
    if args.len() > 1 && args[1] == "-m" {
        println!("UTF-8");
        println!("ASCII");
        println!("ISO-8859-1");
        return;
    }
    let vars = [
        "LANG", "LC_ALL", "LC_COLLATE", "LC_CTYPE", "LC_MESSAGES",
        "LC_MONETARY", "LC_NUMERIC", "LC_TIME",
    ];
    for v in vars {
        match std::env::var(v) {
            Ok(val) => println!("{}={}", v, val),
            Err(_) => println!("{}=\"\"", v),
        }
    }
}
