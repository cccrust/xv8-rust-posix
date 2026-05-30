fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: type <name>...");
        std::process::exit(1);
    }
    let builtins = [
        "alias", "bg", "cd", "command", "echo", "eval", "exec", "exit",
        "export", "fg", "hash", "jobs", "kill", "pwd", "read", "readonly",
        "return", "set", "shift", "test", "times", "trap", "type", "ulimit",
        "umask", "unalias", "wait",
    ];
    let paths = std::env::var("PATH").unwrap_or_default();
    let dirs: Vec<&str> = paths.split(':').collect();

    for arg in &args[1..] {
        if builtins.contains(&arg.as_str()) {
            println!("{} is a shell builtin", arg);
        } else if let Some(found) = find_in_path(arg, &dirs) {
            println!("{} is {}", arg, found);
        } else {
            println!("{} not found", arg);
            std::process::exit(1);
        }
    }
}

fn find_in_path(name: &str, dirs: &[&str]) -> Option<String> {
    for dir in dirs {
        let p = format!("{}/{}", dir, name);
        if std::path::Path::new(&p).is_file() {
            return Some(p);
        }
    }
    None
}
