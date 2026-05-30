fn main() {
    let args: Vec<String> = std::env::args().collect();
    let paths = std::env::var("PATH").unwrap_or_default();
    let dirs: Vec<&str> = paths.split(':').collect();

    if args.len() == 1 {
        // Report all known utilities
        let builtins = [
            "alias", "bg", "cd", "command", "echo", "eval", "exec", "exit",
            "export", "fg", "hash", "jobs", "kill", "pwd", "read", "readonly",
            "return", "set", "shift", "test", "times", "trap", "type", "ulimit",
            "umask", "unalias", "wait",
        ];
        for cmd in &builtins {
            println!("builtin {}", cmd);
        }
        return;
    }

    for arg in &args[1..] {
        let found = find_in_path(arg, &dirs);
        match found {
            Some(path) => println!("{}", path),
            None => {
                eprintln!("hash: {}: not found", arg);
                std::process::exit(1);
            }
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
