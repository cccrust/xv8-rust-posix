fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: patch [file] < patchfile");
        std::process::exit(1);
    }

    // Simplified: read patch from stdin, apply to file
    let fname = &args[1];
    let content = std::fs::read_to_string(fname).unwrap_or_else(|e| {
        eprintln!("patch: cannot read '{}': {}", fname, e);
        std::process::exit(1);
    });

    let patch_content = std::fs::read_to_string(&args[2]).unwrap_or_else(|_| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap_or(0);
        input
    });

    // Very simple: apply unified diff hunks
    let mut lines: Vec<&str> = content.lines().collect();
    let patch_lines: Vec<&str> = patch_content.lines().collect();
    let mut i = 0;

    while i < patch_lines.len() {
        let line = patch_lines[i];
        if line.starts_with("@@") {
            // Parse hunk header: @@ -start,count +start,count @@
            i += 1;
            if i >= patch_lines.len() { break; }
            // Skip context lines and apply -/+ changes
            let mut j = 0;
            while i < patch_lines.len() {
                let pl = patch_lines[i];
                if pl.starts_with("---") || pl.starts_with("+++") || pl.starts_with("@@") {
                    break;
                }
                if pl.starts_with('-') {
                    // Remove line
                    let target = &pl[1..];
                    if j < lines.len() && lines[j].trim() == target.trim() {
                        lines.remove(j);
                    } else {
                        j += 1;
                    }
                } else if pl.starts_with('+') {
                    lines.insert(j, &pl[1..]);
                    j += 1;
                } else if pl.starts_with(' ') {
                    j += 1;
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    std::fs::write(fname, lines.join("\n")).unwrap_or_else(|e| {
        eprintln!("patch: cannot write '{}': {}", fname, e);
        std::process::exit(1);
    });
}
