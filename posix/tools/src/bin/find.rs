use std::fs;
use std::path::Path;

struct FindOpts {
    name: Option<String>,
    type_filter: Option<u8>,
}

fn find_dir(path: &Path, opts: &FindOpts) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let mut matched = true;
            if let Some(pattern) = &opts.name {
                if !name.contains(pattern) { matched = false; }
            }
            if let Some(t) = opts.type_filter {
                let ft = entry.file_type().ok();
                let is_match = match t {
                    b'f' => ft.map(|x| x.is_file()).unwrap_or(false),
                    b'd' => ft.map(|x| x.is_dir()).unwrap_or(false),
                    b'l' => ft.map(|x| x.is_symlink()).unwrap_or(false),
                    _ => true,
                };
                if !is_match { matched = false; }
            }
            if matched {
                println!("{}", p.display());
            }
            if p.is_dir() && !entry.file_type().map(|x| x.is_symlink()).unwrap_or(false) {
                find_dir(&p, opts);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = FindOpts { name: None, type_filter: None };
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        match args[i].as_str() {
            "-name" => { i += 1; if i < args.len() { opts.name = Some(args[i].clone()); } }
            "-type" => { i += 1; if i < args.len() { opts.type_filter = args[i].as_bytes().first().copied(); } }
            _ => { eprintln!("find: invalid option -- '{}'", args[i]); std::process::exit(1); }
        }
        i += 1;
    }

    let paths: Vec<&str> = if i < args.len() { args[i..].iter().map(String::as_str).collect() } else { vec!["."] };

    for p in &paths {
        find_dir(Path::new(p), &opts);
    }
}
