use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut targets: Vec<String> = Vec::new();
    let mut makefile = "Makefile".to_string();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => { i += 1; makefile = args[i].clone(); }
            arg => { targets.push(arg.to_string()); }
        }
        i += 1;
    }

    let content = fs::read_to_string(&makefile).unwrap_or_default();
    let mut rules: Vec<Rule> = Vec::new();
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut cur_recipe: Vec<String> = Vec::new();
    let mut cur_targets: Vec<String> = Vec::new();
    let mut cur_deps: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line.starts_with('\t') || line.starts_with(' ') {
            if !cur_targets.is_empty() {
                cur_recipe.push(line.trim().to_string());
            }
            continue;
        }
        // Flush previous rule
        if !cur_targets.is_empty() && !cur_recipe.is_empty() {
            rules.push(Rule { targets: cur_targets.clone(), deps: cur_deps.clone(), recipe: cur_recipe.clone() });
        }
        cur_targets.clear();
        cur_deps.clear();
        cur_recipe.clear();

        if let Some(eq) = line.find('=') {
            let name = line[..eq].trim().to_string();
            let val = line[eq+1..].trim().to_string();
            vars.insert(name, val);
            continue;
        }
        if let Some(colon) = line.find(':') {
            let targs: Vec<String> = line[..colon].split_whitespace().map(|s| expand(s, &vars)).collect();
            let deps: Vec<String> = line[colon+1..].split_whitespace().map(|s| expand(s, &vars)).collect();
            cur_targets = targs;
            cur_deps = deps;
        }
    }
    if !cur_targets.is_empty() && !cur_recipe.is_empty() {
        rules.push(Rule { targets: cur_targets, deps: cur_deps, recipe: cur_recipe });
    }

    // Expand variables in recipes
    for r in &mut rules {
        for cmd in &mut r.recipe {
            *cmd = expand(cmd, &vars);
        }
    }

    if targets.is_empty() {
        if let Some(first) = rules.first() {
            targets.push(first.targets[0].clone());
        } else {
            eprintln!("make: *** No targets.  Stop.");
            std::process::exit(1);
        }
    }

    for t in &targets {
        build(t, &rules, &mut HashMap::new());
    }
}

#[derive(Clone)]
struct Rule {
    targets: Vec<String>,
    deps: Vec<String>,
    recipe: Vec<String>,
}

fn build(target: &str, rules: &[Rule], visited: &mut HashMap<String, bool>) -> bool {
    if visited.contains_key(target) {
        return visited[target];
    }
    visited.insert(target.to_string(), false);

    let rule = match rules.iter().find(|r| r.targets.iter().any(|t| t == target)) {
        Some(r) => r,
        None => {
            // Implicit rule: target is a file with no deps
            return true;
        }
    };

    let mut need_build = false;
    for dep in &rule.deps {
        build(dep, rules, visited);
        if !is_newer(target, dep) {
            need_build = true;
        }
    }

    // If target doesn't exist, need build
    if !std::path::Path::new(target).exists() {
        need_build = true;
    }

    // .PHONY check: if target has no file, always build
    if rule.deps.is_empty() && !std::path::Path::new(target).exists() {
        need_build = true;
    }

    if need_build {
        for cmd in &rule.recipe {
            let cmd = cmd.trim();
            if cmd.is_empty() { continue; }
            let expanded = cmd
                .replace("$@", target)
                .replace("$^", &rule.deps.join(" "));
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(&expanded)
                .status();
            match status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    eprintln!("make: *** [{}] Error {}", target, s.code().unwrap_or(1));
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("make: *** [{}] {}", target, e);
                    std::process::exit(1);
                }
            }
        }
    }

    visited.insert(target.to_string(), true);
    true
}

fn is_newer(target: &str, dep: &str) -> bool {
    let t_mtime = mtime(target);
    let d_mtime = mtime(dep);
    match (t_mtime, d_mtime) {
        (Some(t), Some(d)) => t >= d,
        (Some(_), None) => true,
        _ => false,
    }
}

fn mtime(path: &str) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn expand(s: &str, vars: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.next() {
                Some('$') => out.push('$'),
                Some('(') => {
                    let mut name = String::new();
                    while let Some(&n) = chars.peek() {
                        if n == ')' { chars.next(); break; }
                        name.push(chars.next().unwrap());
                    }
                    out.push_str(vars.get(&name).map(|s| s.as_str()).unwrap_or(""));
                }
                Some(c) if c.is_alphanumeric() || c == '_' => {
                    let mut name = String::from(c);
                    while let Some(&n) = chars.peek() {
                        if n.is_alphanumeric() || n == '_' { name.push(chars.next().unwrap()); }
                        else { break; }
                    }
                    out.push_str(vars.get(&name).map(|s| s.as_str()).unwrap_or(""));
                }
                _ => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}
