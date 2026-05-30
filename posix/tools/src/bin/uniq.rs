use std::io::BufRead;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut count = false;
    let mut repeated = false;
    let mut unique = false;
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'c' => count = true,
                'd' => repeated = true,
                'u' => unique = true,
                _ => { eprintln!("uniq: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let input: Box<dyn BufRead> = if i < args.len() {
        Box::new(std::io::BufReader::new(std::fs::File::open(&args[i]).unwrap_or_else(|e| {
            eprintln!("uniq: {}: {}", args[i], e);
            std::process::exit(1);
        })))
    } else {
        Box::new(std::io::stdin().lock())
    };

    let mut lines: Vec<String> = Vec::new();
    for line in input.lines() {
        lines.push(line.unwrap_or_default());
    }

    let mut prev: Option<String> = None;
    let mut run_count = 0usize;

    for line in &lines {
        if prev.as_deref() == Some(line.as_str()) {
            run_count += 1;
            continue;
        }
        if let Some(p) = prev.take() {
            emit(&p, run_count, count, repeated, unique);
        }
        prev = Some(line.clone());
        run_count = 1;
    }
    if let Some(p) = prev {
        emit(&p, run_count, count, repeated, unique);
    }
}

fn emit(line: &str, count: usize, show_count: bool, repeated: bool, unique: bool) {
    let is_rep = count > 1;
    if (repeated && !is_rep) || (unique && is_rep) { return; }
    if show_count {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}
