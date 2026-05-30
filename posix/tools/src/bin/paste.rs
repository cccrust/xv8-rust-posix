use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut delim = '\t';
    let mut serial = false;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-d" if i + 1 < args.len() => {
                delim = args[i + 1].chars().next().unwrap_or('\t');
                i += 2;
            }
            "-s" => { serial = true; i += 1; }
            _ => { i += 1; }
        }
    }
    let files: Vec<&str> = if i < args.len() { args[i..].iter().map(|s| s.as_str()).collect() } else { vec!["-"] };

    let mut all_lines: Vec<Vec<String>> = Vec::new();
    for f in &files {
        let lines = read_lines(f);
        all_lines.push(lines);
    }

    if serial {
        let max = all_lines.iter().map(|v| v.len()).max().unwrap_or(0);
        for i in 0..max {
            let mut parts = Vec::new();
            for lines in &all_lines {
                if i < lines.len() {
                    parts.push(lines[i].clone());
                }
            }
            println!("{}", parts.join(&delim.to_string()));
        }
    } else {
        if all_lines.is_empty() { return; }
        let num_files = all_lines.len();

        let mut iterators: Vec<usize> = vec![0; num_files];
        let mut done = false;
        while !done {
            let mut parts = Vec::new();
            for (fi, lines) in all_lines.iter().enumerate() {
                if iterators[fi] < lines.len() {
                    parts.push(lines[iterators[fi]].clone());
                }
            }
            if !parts.is_empty() {
                println!("{}", parts.join(&delim.to_string()));
            }
            done = true;
            for fi in 0..num_files {
                if iterators[fi] < all_lines[fi].len() {
                    iterators[fi] += 1;
                }
                if iterators[fi] < all_lines[fi].len() {
                    done = false;
                }
            }
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
