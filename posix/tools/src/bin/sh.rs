use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio, exit};

const BUILTIN_NAMES: [&str; 5] = ["cd", "exit", "export", "echo", "type"];

#[derive(Clone, Debug)]
struct Redirect {
    fd: u32,
    op: String,
    target: String,
    heredoc_content: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let content = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("sh: cannot open '{}': {}", args[1], e);
            exit(1);
        });
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut idx = 0;
        let mut last_status = 0;
        while idx < lines.len() {
            let trimmed = lines[idx].trim().to_string();
            idx += 1;
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            idx = exec_line(&trimmed, &lines, idx, &args, &mut last_status);
        }
        exit(last_status);
    }

    if io::stdin().is_terminal() {
        repl(&args);
    } else {
        let lines: Vec<String> = io::stdin().lines().filter_map(|l| l.ok()).collect();
        let mut idx = 0;
        let mut last_status = 0;
        while idx < lines.len() {
            let trimmed = lines[idx].trim().to_string();
            idx += 1;
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            idx = exec_line(&trimmed, &lines, idx, &args, &mut last_status);
        }
        exit(last_status);
    }
}

fn repl(args: &[String]) {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().ok();
        input.clear();
        if io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) {
            println!();
            break;
        }
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() { continue; }
        if trimmed == "exit" { break; }
        // REPL mode: no pre-collected lines for heredoc; read extra on the fly
        let empty_lines: Vec<String> = Vec::new();
        let mut last_status = 0;
        let idx = exec_line(&trimmed, &empty_lines, 0, args, &mut last_status);
        // If heredoc consumed lines from empty_lines (idx advanced), that means
        // heredoc wasn't satisfied — read from stdin instead
        if idx > 0 {
            // Need to read heredoc content from stdin
            // Re-parse to find the << delimiter
            let tokens = tokenize(&trimmed);
            let heredoc_delim = tokens.iter()
                .position(|t| t == "<<")
                .and_then(|p| tokens.get(p + 1).cloned());
            if let Some(delim) = heredoc_delim {
                let mut content = String::new();
                loop {
                    print!("> ");
                    io::stdout().flush().ok();
                    input.clear();
                    if io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) { break; }
                    let line = input.trim_end().to_string();
                    if line.trim() == delim { break; }
                    content.push_str(&line);
                    content.push('\n');
                }
                let heredoc_cmd = format!("{} << {}\n{}", trim_before_heredoc(&trimmed), delim, content);
                let heredoc_lines = vec![heredoc_cmd.clone() + &delim];
                let mut ls = 0;
                exec_line(&heredoc_cmd, &heredoc_lines, 1, args, &mut ls);
            }
        }
    }
}

fn trim_before_heredoc(line: &str) -> String {
    let tokens = tokenize(line);
    let mut out = Vec::new();
    for t in &tokens {
        if t == "<<" { break; }
        out.push(t.clone());
    }
    out.join(" ")
}

fn exec_line(line: &str, lines: &[String], line_idx: usize, _args: &[String], last_status: &mut i32) -> usize {
    let tokens = tokenize(line);
    if tokens.is_empty() { return line_idx; }

    let mut idx = line_idx;
    let mut groups: Vec<(Vec<String>, &str)> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut op = "";

    for t in &tokens {
        match t.as_str() {
            ";" | "&&" | "||" => {
                if !cur.is_empty() {
                    groups.push((cur.clone(), op));
                    cur.clear();
                }
                op = t;
            }
            _ => { cur.push(t.clone()); }
        }
    }
    if !cur.is_empty() {
        groups.push((cur, op));
    }

    if groups.is_empty() { return idx; }

    // Execute first group
    let first_op = groups[0].1;
    if first_op == "" || *last_status == 0 && first_op == "&&" || *last_status != 0 && first_op == "||" {
        idx = exec_group(&groups[0].0, lines, idx, last_status);
    }

    for i in 1..groups.len() {
        let (ref tokens, op) = groups[i];
        let should_run = match op {
            "&&" => *last_status == 0,
            "||" => *last_status != 0,
            _ => true,
        };
        if should_run {
            idx = exec_group(tokens, lines, idx, last_status);
        }
    }

    idx
}

fn exec_group(tokens: &[String], lines: &[String], line_idx: usize, last_status: &mut i32) -> usize {
    if tokens.is_empty() { return line_idx; }
    let mut idx = line_idx + 0;

    let background = tokens.last().map_or(false, |t| t == "&");
    let cmd_tokens: Vec<String> = if background {
        tokens[..tokens.len() - 1].to_vec()
    } else {
        tokens.to_vec()
    };

    let (redirects, heredoc_idx) = parse_redirects(&cmd_tokens, lines, idx, background);
    idx = heredoc_idx;

    let argv: Vec<String> = cmd_tokens.iter()
        .filter(|t| {
            !matches!(t.as_str(), "<" | ">" | ">>" | "<<" | "2>" | "2>&1")
            && !is_redirect_target(&cmd_tokens, t)
        })
        .cloned()
        .collect();

    if argv.is_empty() { return idx; }

    let mut env_vars: Vec<(String, String)> = Vec::new();
    let mut cmd_start = 0;
    for arg in &argv {
        if let Some(eq) = arg.find('=') {
            if eq > 0 {
                env_vars.push((arg[..eq].to_string(), arg[eq + 1..].to_string()));
                cmd_start += 1;
                continue;
            }
        }
        break;
    }
    if cmd_start >= argv.len() { return idx; }

    let cmd = &argv[cmd_start];
    let cmd_args: Vec<String> = argv[cmd_start + 1..].to_vec();

    // Builtins (only with direct execution if no redirects)
    // Determine if this is a pipeline (in argv, before redirect filtering)
    let pipe_pos = argv.iter().position(|t| t == "|");
    if let Some(_pp) = pipe_pos {
        let parts: Vec<Vec<String>> = argv.split(|t| t == "|").map(|s| s.to_vec()).collect();
        let segments: Vec<&[String]> = parts.iter().map(|p| p.as_slice()).collect();
        *last_status = exec_pipeline_ext(&segments, &redirects, background);
        return idx;
    }

    // Builtins (only with direct execution if no redirects)
    if redirects.is_empty() {
        if let Some(status) = try_builtin(cmd, &cmd_args) {
            *last_status = status;
            return idx;
        }
    }

    // Single command
    let mut cmd_obj = Command::new(cmd);
    cmd_obj.args(&cmd_args);
    apply_redirects(&mut cmd_obj, &redirects);
    for (k, v) in &env_vars { cmd_obj.env(k, v); }

    if background {
        match cmd_obj.spawn() {
            Ok(c) => {
                let pid = c.id();
                println!("[1] {}", pid);
                *last_status = 0;
            }
            Err(e) => {
                eprintln!("sh: {}: {}", cmd, e);
                *last_status = 127;
            }
        }
    } else {
        match cmd_obj.status() {
            Ok(s) => { *last_status = s.code().unwrap_or(0); }
            Err(e) => {
                eprintln!("sh: {}: {}", cmd, e);
                *last_status = 127;
            }
        }
    }

    idx
}

fn is_redirect_target(tokens: &[String], t: &str) -> bool {
    for i in 0..tokens.len() {
        if tokens[i] == t && i > 0 {
            let prev = tokens[i - 1].as_str();
            if matches!(prev, "<" | ">" | ">>" | "<<" | "2>" | "2>&1") {
                return true;
            }
        }
    }
    false
}

fn parse_redirects(tokens: &[String], _lines: &[String], mut line_idx: usize, _background: bool) -> (Vec<Redirect>, usize) {
    let mut redirects = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let (fd, op, advance) = match tokens[i].as_str() {
            "<" => (0, "<", 1),
            ">" => (1, ">", 1),
            ">>" => (1, ">>", 1),
            "<<" => (0, "<<", 1),
            "2>" => (2, ">", 1),
            "2>&1" => (2, ">&1", 1),
            _ => { i += 1; continue; }
        };

        if tokens[i].as_str() == "<<" {
            // Heredoc
            if i + 1 < tokens.len() {
                let delim = tokens[i + 1].clone();
                let mut content = String::new();
                // Read lines until delimiter
                while line_idx < _lines.len() {
                    let l = _lines[line_idx].trim_end().to_string();
                    line_idx += 1;
                    if l.trim() == delim { break; }
                    content.push_str(&l);
                    content.push('\n');
                }
                redirects.push(Redirect {
                    fd: 0,
                    op: "<<".to_string(),
                    target: String::new(),
                    heredoc_content: Some(content),
                });
            }
            i += 2;
            continue;
        }

        if i + 1 < tokens.len() {
            redirects.push(Redirect {
                fd,
                op: op.to_string(),
                target: tokens[i + 1].clone(),
                heredoc_content: None,
            });
        }
        i += 1 + advance;
    }

    (redirects, line_idx)
}

fn apply_redirects(cmd: &mut Command, redirects: &[Redirect]) {
    for r in redirects {
        match r.op.as_str() {
            "<" => {
                if let Ok(f) = fs::File::open(&r.target) {
                    cmd.stdin(f);
                }
            }
            ">" => {
                if let Ok(f) = fs::File::create(&r.target) {
                    match r.fd {
                        1 => { cmd.stdout(f); }
                        2 => { cmd.stderr(f); }
                        _ => {}
                    }
                }
            }
            ">>" => {
                if let Ok(f) = fs::OpenOptions::new().create(true).append(true).open(&r.target) {
                    cmd.stdout(f);
                }
            }
            ">&1" => {
                // Dup to stdout (for 2>&1 etc.)
                // Use Stdio::inherit() for the target fd
                if r.fd == 2 {
                    cmd.stderr(Stdio::inherit());
                }
            }
            "<<" => {
                if let Some(ref content) = r.heredoc_content {
                    let temp = format!("/tmp/sh_heredoc_{}_{}", std::process::id(), std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_nanos());
                    let _ = fs::write(&temp, content);
                    if let Ok(f) = fs::File::open(&temp) {
                        cmd.stdin(f);
                    }
                }
            }
            _ => {}
        }
    }
}

fn exec_pipeline_ext(segments: &[&[String]], global_redirects: &[Redirect], background: bool) -> i32 {
    if segments.is_empty() { return 0; }
    let mut children: Vec<std::process::Child> = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;

    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() { continue; }
        let (seg_redirects, _) = parse_redirects(seg, &[], 0, false);
        let argv: Vec<String> = seg.iter()
            .filter(|t| !matches!(t.as_str(), "<" | ">" | ">>" | "<<" | "2>" | "2>&1"))
            .filter(|t| !is_redirect_target_any(seg, t))
            .cloned()
            .collect();

        if argv.is_empty() { continue; }

        let mut cmd = Command::new(&argv[0]);
        cmd.args(&argv[1..]);

        if let Some(prev) = prev_stdout.take() {
            cmd.stdin(prev);
        }

        // Apply segment-level redirects
        apply_redirects(&mut cmd, &seg_redirects);

        // For the last segment, apply global redirects
        if i == segments.len() - 1 {
            apply_redirects(&mut cmd, global_redirects);
        }

        if i < segments.len() - 1 {
            cmd.stdout(Stdio::piped());
        }

        match cmd.spawn() {
            Ok(mut c) => {
                prev_stdout = c.stdout.take();
                children.push(c);
            }
            Err(e) => {
                eprintln!("sh: {}: {}", argv[0], e);
                for mut c in children { let _ = c.wait(); }
                return 127;
            }
        }
    }

    if background {
        if let Some(c) = children.last() {
            println!("[1] {}", c.id());
        }
        return 0;
    }

    let mut status = 0;
    for mut child in children {
        status = child.wait().map(|s| s.code().unwrap_or(0)).unwrap_or(0);
    }
    status
}

fn is_redirect_target_any(tokens: &[String], t: &str) -> bool {
    for i in 0..tokens.len() {
        if tokens[i] == t && i > 0 {
            let prev = tokens[i - 1].as_str();
            if matches!(prev, "<" | ">" | ">>" | "<<" | "2>" | "2>&1") {
                return true;
            }
        }
    }
    false
}

fn try_builtin(cmd: &str, args: &[String]) -> Option<i32> {
    match cmd {
        "cd" => {
            let dir = if args.is_empty() {
                env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else {
                args[0].clone()
            };
            if let Err(e) = env::set_current_dir(Path::new(&dir)) {
                eprintln!("cd: {}: {}", dir, e);
                Some(1)
            } else {
                Some(0)
            }
        }
        "exit" => {
            let code = args.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            exit(code);
        }
        "export" => {
            for arg in args {
                if let Some(eq) = arg.find('=') {
                    env::set_var(&arg[..eq], &arg[eq + 1..]);
                }
            }
            Some(0)
        }
        "echo" => {
            println!("{}", args.join(" "));
            Some(0)
        }
        "type" => {
            if args.is_empty() {
                eprintln!("type: usage: type name ...");
                return Some(1);
            }
            for arg in args {
                if BUILTIN_NAMES.contains(&arg.as_str()) {
                    println!("{} is a shell builtin", arg);
                } else if let Ok(path) = which(arg) {
                    println!("{} is {}", arg, path);
                } else {
                    println!("{}: not found", arg);
                }
            }
            Some(0)
        }
        _ => None,
    }
}

fn which(name: &str) -> Result<String, ()> {
    let path = env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        let full = Path::new(dir).join(name);
        if full.is_file() {
            return Ok(full.to_string_lossy().to_string());
        }
    }
    Err(())
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut in_dquote = false;
    let mut escape = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    let flush = |current: &mut String, tokens: &mut Vec<String>| {
        if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }
    };

    while i < chars.len() {
        let c = chars[i];
        if escape {
            current.push(c);
            escape = false;
            i += 1;
            continue;
        }
        if c == '\\' && !in_quote {
            escape = true;
            i += 1;
            continue;
        }
        if c == '\'' && !in_dquote {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if c == '"' && !in_quote {
            in_dquote = !in_dquote;
            i += 1;
            continue;
        }
        if (c == ' ' || c == '\t') && !in_quote && !in_dquote {
            flush(&mut current, &mut tokens);
            i += 1;
            continue;
        }

        if !in_quote && !in_dquote {
            match c {
                '|' => { flush(&mut current, &mut tokens); tokens.push("|".to_string()); i += 1; continue; }
                ';' => { flush(&mut current, &mut tokens); tokens.push(";".to_string()); i += 1; continue; }
                '&' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == '&' {
                        tokens.push("&&".to_string()); i += 2;
                    } else {
                        tokens.push("&".to_string()); i += 1;
                    }
                    continue;
                }
                '<' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == '<' {
                        tokens.push("<<".to_string()); i += 2;
                    } else {
                        tokens.push("<".to_string()); i += 1;
                    }
                    continue;
                }
                '>' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(">>".to_string()); i += 2;
                    } else {
                        tokens.push(">".to_string()); i += 1;
                    }
                    continue;
                }
                _ => {}
            }
            // Handle 2>, 2>&1 (digit followed by > or >&)
            if c.is_ascii_digit() && i + 1 < chars.len() && chars[i + 1] == '>' {
                flush(&mut current, &mut tokens);
                if i + 2 < chars.len() && chars[i + 1] == '>' && chars[i + 2] == '>' {
                    // 2>>
                    current.push(c); current.push('>'); current.push('>');
                    tokens.push(current.clone()); current.clear();
                    i += 3;
                } else if i + 2 < chars.len() && chars[i + 1] == '>' && chars[i + 2] == '&' && i + 3 < chars.len() && chars[i + 3] == '1' {
                    // 2>&1
                    tokens.push(format!("{}>&1", c));
                    i += 4;
                } else {
                    // 2>
                    tokens.push(format!("{}>", c));
                    i += 2;
                }
                continue;
            }
        }

        // Variable expansion
        if c == '$' && !in_quote {
            i += 1;
            let mut var = String::new();
            if i < chars.len() && chars[i] == '{' {
                i += 1;
                while i < chars.len() && chars[i] != '}' {
                    var.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() { i += 1; } // skip }
                // Handle :-default
                if let Some(col_idx) = var.find(":-") {
                    let name = &var[..col_idx];
                    let default = &var[col_idx + 2..];
                    let val = env::var(name).unwrap_or_else(|_| default.to_string());
                    current.push_str(&val);
                } else {
                    let val = env::var(&var).unwrap_or_default();
                    current.push_str(&val);
                }
            } else {
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    var.push(chars[i]);
                    i += 1;
                }
                let val = if var.is_empty() {
                    // $? = last exit status (use 0 as fallback)
                    if i > 0 && chars[i-1] == '?' {
                        "0".to_string()
                    } else {
                        String::new()
                    }
                } else {
                    env::var(&var).unwrap_or_default()
                };
                current.push_str(&val);
            }
            continue;
        }

        current.push(c);
        i += 1;
    }
    flush(&mut current, &mut tokens);
    tokens
}
