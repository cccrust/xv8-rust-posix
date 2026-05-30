use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut prefix = "x".to_string();
    let mut suffix_len = 2;
    let mut lines_per_file = 1000;
    let _numeric = false;

    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-a" if i + 1 < args.len() => { suffix_len = args[i + 1].parse().unwrap_or(2); i += 2; }
            "-b" | "-l" if i + 1 < args.len() => {
                let s = args[i + 1].clone();
                if s.ends_with('k') {
                    lines_per_file = s[..s.len()-1].parse::<usize>().unwrap_or(1000) * 1000;
                } else {
                    lines_per_file = s.parse().unwrap_or(1000);
                }
                i += 2;
            }
            "-d" if i + 1 < args.len() => { prefix = args[i + 1].clone(); i += 2; }
            _ => { i += 1; }
        }
    }
    let file = if i < args.len() { args[i].as_str() } else { "-" };

    let lines = read_lines(file);
    let num_files = (lines.len() + lines_per_file - 1) / lines_per_file;
    let digits = num_files.to_string().len().max(suffix_len);

    let mut chunk = 0;
    let mut out_lines = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        out_lines.push(line);
        if out_lines.len() == lines_per_file || idx == lines.len() - 1 {
            let suffix = format!("{:0width$}", chunk, width = digits);
            let filename = format!("{}{}", prefix, suffix);
            let mut f = std::fs::File::create(&filename).unwrap_or_else(|_| {
                eprintln!("split: cannot create {}", filename);
                std::process::exit(1);
            });
            for l in &out_lines {
                writeln!(f, "{}", l).ok();
            }
            out_lines.clear();
            chunk += 1;
        }
    }
}

fn read_lines(path: &str) -> Vec<String> {
    if path == "-" {
        io::stdin().lock().lines().filter_map(|l| l.ok()).collect()
    } else {
        std::fs::read_to_string(path).unwrap_or_default().lines().map(|l| l.to_string()).collect()
    }
}
