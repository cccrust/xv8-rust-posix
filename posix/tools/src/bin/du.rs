use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut summarize = false;
    let mut human = false;
    let mut max_depth: Option<usize> = None;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                's' => summarize = true,
                'h' => human = true,
                'd' => { i += 1; if i < args.len() { max_depth = args[i].parse().ok(); } }
                _ => { eprintln!("du: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let target = if i < args.len() { Path::new(&args[i]) } else { Path::new(".") };

    du_dir(target, 0, max_depth, human, summarize);
}

fn du_dir(path: &Path, depth: usize, max_depth: Option<usize>, human: bool, summarize: bool) -> u64 {
    if let Some(md) = max_depth {
        if depth > md { return 0; }
    }

    let mut total = 0u64;
    let is_dir = path.is_dir();

    if is_dir {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                total += du_dir(&entry.path(), depth + 1, max_depth, human, summarize);
            }
        }
    }

    // Add own size
    if let Ok(meta) = path.metadata() {
        total += meta.len();
    }

    if depth == 0 || (!summarize && depth > 0) || (summarize && depth == 0) {
        if human {
            println!("{}\t{}", format_size(total), path.display());
        } else {
            println!("{}\t{}", total, path.display());
        }
    }

    total
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[char] = &['K', 'M', 'G', 'T'];
    let mut size = bytes as f64;
    for &unit in UNITS {
        if size >= 1024.0 {
            size /= 1024.0;
            if size < 1024.0 {
                return format!("{:.1}{}", size, unit);
            }
        }
    }
    format!("{:.1}P", size)
}
