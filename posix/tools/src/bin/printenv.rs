fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // Print all environment variables
        for (key, val) in std::env::vars() {
            println!("{}={}", key, val);
        }
    } else {
        for var in &args[1..] {
            match std::env::var(var) {
                Ok(val) => println!("{}", val),
                Err(_) => {} // Silently ignore as per POSIX
            }
        }
    }
}
