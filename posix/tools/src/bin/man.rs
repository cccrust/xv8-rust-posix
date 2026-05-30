fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut section = String::new();
    let mut name = String::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-k" => {
                i += 1;
                let kw = if i < args.len() { &args[i] } else { "" };
                search_man(kw);
                return;
            }
            a if a.starts_with('-') => { i += 1; }
            a => {
                if name.is_empty() {
                    if let Some(dot) = a.find('.') {
                        section = a[dot+1..].to_string();
                        name = a[..dot].to_string();
                    } else if a.chars().all(|c| c.is_ascii_digit()) {
                        section = a.to_string();
                    } else {
                        name = a.to_string();
                    }
                } else if section.is_empty() && a.chars().all(|c| c.is_ascii_digit()) {
                    section = a.to_string();
                }
                i += 1;
            }
        }
    }
    if name.is_empty() {
        eprintln!("What manual page do you want?");
        std::process::exit(1);
    }
    let manpath = std::env::var("MANPATH").unwrap_or_else(|_| "/usr/share/man:/usr/local/share/man".to_string());
    for dir in manpath.split(':') {
        let secs = if section.is_empty() {
            vec!["1", "2", "3", "4", "5", "6", "7", "8"]
        } else {
            vec![section.as_str()]
        };
        for sec in &secs {
            let paths = [
                format!("{}/{}/{}.{}", dir, sec, name, sec),
                format!("{}/man{}/{}.{}", dir, sec, name, sec),
            ];
            for p in &paths {
                if std::path::Path::new(p).exists() {
                    display_man(p);
                    return;
                }
            }
        }
    }
    eprintln!("No manual entry for {}", name);
    std::process::exit(1);
}

fn search_man(kw: &str) {
    let manpath = std::env::var("MANPATH").unwrap_or_else(|_| "/usr/share/man:/usr/local/share/man".to_string());
    for dir in manpath.split(':') {
        let base = std::path::Path::new(dir);
        if !base.is_dir() { continue; }
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let _fname = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                if path.is_dir() { continue; }
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let name_lower = stem.to_lowercase();
                let kw_lower = kw.to_lowercase();
                if name_lower.contains(&kw_lower) {
                    let p = entry.path();
                    let sec = p.parent().and_then(|p| {
                        p.file_name().and_then(|s| s.to_str())
                    }).unwrap_or("");
                    let sec = sec.trim_start_matches("man");
                    println!("{} ({})  -", stem, sec);
                }
            }
        }
    }
}

fn display_man(path: &str) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    // Try using mandoc or nroff first
    for fmt in &["mandoc", "nroff", "groff"] {
        if let Ok(child) = std::process::Command::new(fmt)
            .arg("-mandoc")
            .arg(path)
            .stdout(std::process::Stdio::piped())
            .spawn()
        {
            if let Ok(output) = child.wait_with_output() {
                if output.status.success() {
                    let formatted = String::from_utf8_lossy(&output.stdout);
                    page_output(&formatted);
                    return;
                }
            }
        }
    }
    // Fallback: raw display with pager
    page_output(&content);
}

fn page_output(text: &str) {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "more".to_string());
    if let Ok(mut child) = std::process::Command::new(&pager)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        use std::io::Write;
        let _ = child.stdin.take().map(|mut s| s.write_all(text.as_bytes()));
        let _ = child.wait();
    } else {
        // Fallback: just print
        println!("{}", text);
    }
}
