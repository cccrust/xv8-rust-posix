use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: basename string [suffix]");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let mut name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => {
            // If path ends with /, file_name returns None
            // Treat as empty or root
            if args[1].ends_with('/') && args[1] != "/" {
                let trimmed = args[1].trim_end_matches('/');
                Path::new(trimmed).file_name().unwrap_or_default().to_string_lossy().to_string()
            } else {
                args[1].clone()
            }
        }
    };

    if args.len() > 2 {
        let suffix = &args[2];
        if name.ends_with(suffix) && !suffix.is_empty() {
            let end = name.len() - suffix.len();
            name.truncate(end);
        }
    }

    println!("{}", name);
}
