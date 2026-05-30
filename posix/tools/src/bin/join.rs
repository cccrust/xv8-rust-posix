use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: join [-t <sep>] [-j <field>] [-1 <field>] [-2 <field>] <file1> <file2>");
        std::process::exit(1);
    }
    let mut i = 1;
    let mut sep: Option<char> = None;
    let mut file1_field = 1usize;
    let mut file2_field = 1usize;
    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-t" if i + 1 < args.len() => {
                sep = args[i + 1].chars().next();
                i += 2;
            }
            "-1" if i + 1 < args.len() => {
                file1_field = args[i + 1].parse().unwrap_or(1);
                i += 2;
            }
            "-2" if i + 1 < args.len() => {
                file2_field = args[i + 1].parse().unwrap_or(1);
                i += 2;
            }
            _ => { i += 1; }
        }
    }
    if i + 2 > args.len() {
        eprintln!("Usage: join <file1> <file2>");
        std::process::exit(1);
    }
    let file1 = &args[i];
    let file2 = &args[i + 1];

    let lines1 = read_lines(file1);
    let lines2 = read_lines(file2);

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in &lines1 {
        let fields = split_fields(line, sep);
        let key = if fields.len() >= file1_field { fields[file1_field - 1].clone() } else { String::new() };
        map.entry(key).or_default().push(line.clone());
    }

    for line in &lines2 {
        let fields = split_fields(line, sep);
        let key = if fields.len() >= file2_field { fields[file2_field - 1].clone() } else { String::new() };
        if let Some(matches) = map.get(&key) {
            for m in matches {
                let mfields = split_fields(m, sep);
                let mut combined = vec![key.clone()];
                for (i, f) in mfields.iter().enumerate() {
                    if i != file1_field - 1 {
                        combined.push(f.clone());
                    }
                }
                for (i, f) in fields.iter().enumerate() {
                    if i != file2_field - 1 {
                        combined.push(f.clone());
                    }
                }
                let out_sep = sep.map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());
                println!("{}", combined.join(&out_sep));
            }
        }
    }
}

fn split_fields(line: &str, sep: Option<char>) -> Vec<String> {
    match sep {
        Some(c) => line.split(c).map(|s| s.to_string()).collect(),
        None => line.split_whitespace().map(|s| s.to_string()).collect(),
    }
}

fn read_lines(path: &str) -> Vec<String> {
    if path == "-" {
        use std::io::BufRead;
        std::io::stdin().lock().lines().filter_map(|l| l.ok()).collect()
    } else {
        std::fs::read_to_string(path).unwrap_or_default().lines().map(|l| l.to_string()).collect()
    }
}
