use std::io::{self, BufRead};
use std::path::Path;

struct GrepOpts {
    ignore_case: bool,
    invert: bool,
    count: bool,
    line_number: bool,
    files_with_matches: bool,
}

fn matches(line: &str, pattern: &str, opts: &GrepOpts) -> bool {
    let matched = if opts.ignore_case {
        line.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        line.contains(pattern)
    };
    matched != opts.invert
}

fn grep_file(path: &Path, pattern: &str, opts: &GrepOpts) -> io::Result<()> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut match_count = 0;

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if matches(&line, pattern, opts) {
            match_count += 1;
            if opts.count {
                continue;
            }
            if opts.files_with_matches {
                println!("{}", path.display());
                return Ok(());
            }
            if opts.line_number {
                print!("{}:", i + 1);
            }
            println!("{}", line);
        }
    }

    if opts.count {
        println!("{}", match_count);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = GrepOpts {
        ignore_case: false,
        invert: false,
        count: false,
        line_number: false,
        files_with_matches: false,
    };
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                'i' => opts.ignore_case = true,
                'v' => opts.invert = true,
                'c' => opts.count = true,
                'n' => opts.line_number = true,
                'l' => opts.files_with_matches = true,
                _ => { eprintln!("grep: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    if i >= args.len() {
        eprintln!("usage: grep [-ivcnl] pattern [file ...]");
        std::process::exit(1);
    }

    let pattern = &args[i];
    i += 1;
    let files: Vec<String> = args[i..].to_vec();

    if files.is_empty() {
        let mut cnt = 0usize;
        for (i, line) in io::stdin().lock().lines().enumerate() {
            let line = line.unwrap_or_default();
            if matches(&line, pattern, &opts) {
                cnt += 1;
                if opts.count { continue; }
                if opts.line_number { print!("{}:", i + 1); }
                println!("{}", line);
            }
        }
        if opts.count { println!("{}", cnt); }
    } else {
        for fname in &files {
            let path = Path::new(fname);
            if let Err(e) = grep_file(path, pattern, &opts) {
                eprintln!("grep: {}: {}", fname, e);
                std::process::exit(1);
            }
        }
    }
}
