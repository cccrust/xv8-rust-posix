fn main() {
    let args: Vec<String> = std::env::args().collect();
    let s = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "y".to_string()
    };

    loop {
        println!("{}", s);
    }
}
