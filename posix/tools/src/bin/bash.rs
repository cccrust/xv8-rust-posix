use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;

struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Lexer { input: s.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn collect_until<F>(&mut self, mut f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if f(c) {
                break;
            }
            s.push(c);
        }
        s
    }

    fn collect_paren_depth(&mut self) -> String {
        let mut content = String::new();
        let mut depth = 1;
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        while let Some(c) = self.advance() {
            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                content.push(c);
            } else if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                content.push(c);
            } else if c == '(' && !in_single_quote && !in_double_quote {
                depth += 1;
                content.push(c);
            } else if c == ')' && !in_single_quote && !in_double_quote {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                content.push(c);
            } else {
                content.push(c);
            }
        }

        content
    }

    fn collect_brace(&mut self) -> String {
        self.collect_until(|c| c == '}')
    }

    fn parse_variable_name(&mut self) -> String {
        let mut name = String::new();
        if let Some(c) = self.peek() {
            if c == '?' || c == '$' || c == '0' || c == '#'
                || c == '@' || c == '*' || c == '!' || c.is_ascii_digit()
            {
                self.advance();
                name.push(c);
            } else {
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        name.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        name
    }
}

impl Iterator for Lexer {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }

        let c = self.advance()?;

        match c {
            '\n' => Some("\n".into()),
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                }
                Some("&&".into())
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                }
                Some("||".into())
            }
            ';' => Some(";".into()),
            '(' => Some("(".into()),
            ')' => Some(")".into()),
            '{' => Some("{".into()),
            '}' => Some("}".into()),
            '<' => Some("<".into()),
            '>' => Some(">".into()),
            '$' => {
                if self.peek() == Some('{') {
                    self.advance();
                    let name = self.parse_variable_name();
                    Some(format!("${{{}}}", name))
                } else if self.peek() == Some('(') {
                    self.advance();
                    if self.peek() == Some('(') {
                        self.advance();
                        let expr = self.collect_paren_depth();
                        Some(format!("$(({}))", expr))
                    } else {
                        let cmd = self.collect_paren_depth();
                        Some(format!("$({})", cmd))
                    }
                } else {
                    let name = self.parse_variable_name();
                    if name.is_empty() {
                        Some("$".into())
                    } else {
                        Some(format!("${}", name))
                    }
                }
            }
            '"' => {
                let mut s = String::new();
                loop {
                    match self.peek() {
                        Some('"') => {
                            self.advance();
                            break;
                        }
                        Some('\\') => {
                            self.advance();
                            if let Some(ch) = self.advance() {
                                s.push(ch);
                            }
                        }
                        Some('$') => {
                            self.advance();
                            if self.peek() == Some('{') {
                                self.advance();
                                let name = self.parse_variable_name();
                                s.push_str(&format!("${{{}}}", name));
                            } else if self.peek() == Some('(') {
                                self.advance();
                                if self.peek() == Some('(') {
                                    self.advance();
                                    let expr = self.collect_paren_depth();
                                    s.push_str(&format!("$(({})", expr));
                                } else {
                                    let cmd = self.collect_paren_depth();
                                    s.push_str(&format!("$({})", cmd));
                                }
                            } else {
                                let name = self.parse_variable_name();
                                if name.is_empty() {
                                    s.push('$');
                                } else {
                                    s.push_str(&format!("${}", name));
                                }
                            }
                        }
                        Some(ch) => {
                            s.push(ch);
                            self.advance();
                        }
                        None => break,
                    }
                }
                Some(s)
            }
            '\'' => {
                let mut s = String::new();
                while let Some(ch) = self.advance() {
                    if ch == '\'' {
                        break;
                    }
                    s.push(ch);
                }
                Some(s)
            }
            _ => {
                let mut w = c.to_string();
                while let Some(ch) = self.peek() {
                    if " \t\n;|&(){}<>!".contains(ch) {
                        break;
                    }
                    w.push(ch);
                    self.advance();
                }
                Some(w)
            }
        }
    }
}

fn expand_vars(s: &str, globals: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            if let Some(&'(') = chars.peek() {
                chars.next();
                if let Some(&'(') = chars.peek() {
                    chars.next();
                    let mut depth = 1;
                    let mut expr = String::new();
                    while let Some(c) = chars.next() {
                        if c == '(' {
                            depth += 1;
                            expr.push(c);
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr.push(c);
                        } else {
                            expr.push(c);
                        }
                    }
                    let expanded = expand_vars(&expr, globals);
                    if let Ok(o) = Command::new("sh").arg("-c").arg(&format!("echo $(( {} ))", expanded)).output() {
                        result.push_str(&String::from_utf8_lossy(&o.stdout).trim());
                    }
                } else {
                    let mut depth = 1;
                    let mut cmd = String::new();
                    while let Some(c) = chars.next() {
                        if c == '(' {
                            depth += 1;
                            cmd.push(c);
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            cmd.push(c);
                        } else {
                            cmd.push(c);
                        }
                    }
                    if let Ok(o) = Command::new("sh").arg("-c").arg(&cmd).output() {
                        result.push_str(&String::from_utf8_lossy(&o.stdout).trim());
                    }
                }
            } else if let Some(&'{') = chars.peek() {
                chars.next();
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' || c == ':' || c == ' ' || c == '\n' {
                        break;
                    }
                    name.push(c);
                    chars.next();
                }
                if let Some(&'}') = chars.peek() {
                    chars.next();
                }
                if let Some(&':') = chars.peek() {
                    chars.next();
                    let mut default = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            break;
                        }
                        default.push(c);
                        chars.next();
                    }
                    if let Some(&'}') = chars.peek() {
                        chars.next();
                    }
                    let val = globals.get(&name).cloned().unwrap_or_default();
                    result.push_str(if val.is_empty() { &default } else { &val });
                } else {
                    let val = globals.get(&name).cloned().unwrap_or_default();
                    result.push_str(&val);
                }
            } else {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    result.push('$');
                } else if let Some(val) = globals.get(&name) {
                    result.push_str(val);
                }
            }
        } else if ch == '`' {
            let mut cmd = String::new();
            while let Some(&c) = chars.peek() {
                if c == '`' {
                    chars.next();
                    break;
                }
                cmd.push(c);
                chars.next();
            }
            if let Ok(o) = Command::new("sh").arg("-c").arg(&cmd).output() {
                result.push_str(&String::from_utf8_lossy(&o.stdout).trim());
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn expand_glob(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < cs.len() {
        if cs[i] == '*' || cs[i] == '?' || cs[i] == '[' {
            if let Ok(entries) = fs::read_dir(".") {
                let matches: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                if !matches.is_empty() {
                    result.push_str(&matches.join(" "));
                }
            }
            i += 1;
        } else {
            result.push(cs[i]);
            i += 1;
        }
    }

    result
}

fn expand(s: &str, globals: &HashMap<String, String>) -> Vec<String> {
    let expanded = expand_vars(s, globals);
    let globbed = expand_glob(&expanded);
    globbed.split_whitespace().map(String::from).collect()
}

struct Shell {
    aliases: HashMap<String, String>,
    globals: HashMap<String, String>,
}

impl Shell {
    fn new() -> Self {
        let mut globals: HashMap<String, String> = env::vars().collect();
        globals.insert("HOME".into(), env::var("HOME").unwrap_or_else(|_| "/".into()));
        Shell { aliases: HashMap::new(), globals }
    }

    fn run(&mut self, line: &str) -> i32 {
        let toks: Vec<String> = Lexer::new(line).collect();
        self.exec(&toks)
    }

    fn exec(&mut self, toks: &[String]) -> i32 {
        let mut args = Vec::new();

        for tok in toks {
            match tok.as_str() {
                "&&" => {
                    let code = self.run_args(&args);
                    if code != 0 {
                        return code;
                    }
                    args.clear();
                }
                "||" => {
                    let code = self.run_args(&args);
                    if code == 0 {
                        return code;
                    }
                    args.clear();
                }
                ";" => {
                    self.run_args(&args);
                    args.clear();
                }
                "|" => args.push("|PIPE|".into()),
                _ => args.push(tok.clone()),
            }
        }
        if !args.is_empty() {
            self.run_args(&args)
        } else {
            0
        }
    }

    fn run_args(&mut self, args: &[String]) -> i32 {
        if args.is_empty() {
            return 0;
        }

        let pipe_pos = args.iter().position(|a| a == "|PIPE|");
        if let Some(pp) = pipe_pos {
            let left = self.expand_args(&args[..pp]);
            let right = self.expand_args(&args[pp + 1..]);
            if left.is_empty() || right.is_empty() {
                return 0;
            }
            let out = Command::new(&left[0]).args(&left[1..]).output();
            match out {
                Ok(o) => {
                    let mut child = Command::new(&right[0]).args(&right[1..]).spawn().ok();
                    if let Some(ref mut c) = child {
                        if let Some(ref mut stdin) = c.stdin.take() {
                            stdin.write_all(&o.stdout).ok();
                        }
                    }
                    if let Some(ref mut c) = child {
                        if let Ok(s) = c.wait() {
                            return s.code().unwrap_or(0);
                        }
                    }
                }
                Err(_) => {}
            }
            return 1;
        }

        let expanded = self.expand_args(args);
        if expanded.is_empty() {
            return 0;
        }

        if let Some(first) = expanded.first() {
            if let Some(eq_pos) = first.find('=') {
                let var = &first[..eq_pos];
                let val = &first[eq_pos + 1..];
                if !var.is_empty() && !var.contains('/') && var.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    self.globals.insert(var.to_string(), val.to_string());
                    if expanded.len() == 1 {
                        return 0;
                    }
                    return self.exec_builtin_or_external(&expanded[1], &expanded[2..]);
                }
            }
        }

        let first = &expanded[0];
        let rest: Vec<String> = expanded[1..].to_vec();

        if first == "for" || first == "while" || first == "until" || first == "if" || first == "case" || first == "function" {
            return self.exec_compound(&expanded);
        }

        self.exec_builtin_or_external(first, &rest)
    }

    fn expand_args(&self, args: &[String]) -> Vec<String> {
        args.iter().flat_map(|a| expand(a, &self.globals)).collect()
    }

    fn exec_compound(&mut self, args: &[String]) -> i32 {
        match args[0].as_str() {
            "for" => {
                if args.len() < 2 {
                    return 1;
                }
                let var = &args[1];
                let body_start = args.iter().position(|a| a == "do").unwrap_or(0);
                let done_pos = args.iter().position(|a| a == "done").unwrap_or(args.len());
                let body = &args[body_start + 1..done_pos];

                let items: Vec<String> = if args.len() > 3 && args[2] == "in" {
                    args[3..body_start].to_vec()
                } else {
                    env::args().skip(1).collect()
                };

                for item in items {
                    self.globals.insert(var.clone(), item);
                    for b in body {
                        self.run(b);
                    }
                }
                0
            }
            "while" | "until" => {
                if args.len() < 5 {
                    return 1;
                }
                let body_start = args.iter().position(|a| a == "do").unwrap_or(0);
                let done_pos = args.iter().position(|a| a == "done").unwrap_or(args.len());
                let cond = &args[1..body_start];
                let body = &args[body_start + 1..done_pos];
                let is_until = args[0] == "until";

                loop {
                    let code = self.exec(cond);
                    if (is_until && code == 0) || (!is_until && code != 0) {
                        break;
                    }
                    for b in body {
                        self.run(b);
                    }
                }
                0
            }
            "if" => {
                let fi_pos = args.iter().position(|a| a == "fi").unwrap_or(args.len());
                let then_pos = args.iter().position(|a| a == "then").unwrap_or(0);
                let else_pos = args.iter().position(|a| a == "else" || a == "elif");
                let code = self.exec(&args[1..then_pos]);
                if code == 0 {
                    self.exec(&args[then_pos + 1..else_pos.unwrap_or(fi_pos)]);
                } else if let Some(ep) = else_pos {
                    self.exec(&args[ep + 1..fi_pos]);
                }
                0
            }
            "case" => 0,
            "function" => 0,
            _ => 1,
        }
    }

    fn exec_builtin_or_external(&mut self, cmd: &str, args: &[String]) -> i32 {
        if let Some(code) = self.builtin(cmd, args) {
            return code;
        }

        let mut child = Command::new(cmd);
        child.args(args);
        child.envs(env::vars());

        match child.status() {
            Ok(s) => s.code().unwrap_or(0),
            Err(e) => {
                eprintln!("bash: {}: {}", cmd, e);
                127
            }
        }
    }

    fn builtin(&mut self, cmd: &str, args: &[String]) -> Option<i32> {
        match cmd {
            "echo" => {
                println!("{}", args.join(" "));
                Some(0)
            }
            "exit" => {
                let c = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                std::process::exit(c);
            }
            "cd" => {
                let d = if let Some(s) = args.first() {
                    s.as_str()
                } else {
                    &env::var("HOME").unwrap_or_else(|_| "/".to_string())
                };
                if let Err(e) = env::set_current_dir(Path::new(d)) {
                    eprintln!("cd: {}: {}", d, e);
                    return Some(1);
                }
                Some(0)
            }
            "pwd" => {
                let _ = env::current_dir().map(|p| println!("{}", p.display()));
                Some(0)
            }
            "export" => {
                for a in args {
                    if let Some(eq) = a.find('=') {
                        let (k, v) = a.split_at(eq);
                        env::set_var(k, &v[1..]);
                        self.globals.insert(k.to_string(), v[1..].to_string());
                    } else if let Ok(v) = env::var(a) {
                        println!("export {}={}", a, v);
                    }
                }
                Some(0)
            }
            "local" => {
                for a in args {
                    if let Some(eq) = a.find('=') {
                        let (k, v) = a.split_at(eq);
                        self.globals.insert(k.to_string(), v[1..].to_string());
                    }
                }
                Some(0)
            }
            "unset" => {
                for a in args {
                    self.globals.remove(a);
                    env::remove_var(a);
                }
                Some(0)
            }
            "alias" => {
                if args.is_empty() {
                    for (k, v) in &self.aliases {
                        println!("alias {}={}", k, v);
                    }
                } else {
                    for a in args {
                        if let Some(eq) = a.find('=') {
                            let (k, v) = a.split_at(eq);
                            self.aliases.insert(k.to_string(), v[1..].to_string());
                        }
                    }
                }
                Some(0)
            }
            "unalias" => {
                for a in args {
                    self.aliases.remove(a);
                }
                Some(0)
            }
            "read" => {
                let var = args.first().map(|s| s.as_str()).unwrap_or("REPLY");
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    self.globals.insert(var.to_string(), input.trim_end().to_string());
                    Some(0)
                } else {
                    Some(1)
                }
            }
            "eval" => {
                let code = self.run(&args.join(" "));
                Some(code)
            }
            "true" => Some(0),
            "false" => Some(1),
            "type" => {
                for a in args {
                    if self.aliases.contains_key(a) {
                        println!("{} is an alias", a);
                    } else if which(a).is_some() {
                        println!("{} is {}", a, which(a).unwrap());
                    } else {
                        println!("{}: not found", a);
                    }
                }
                Some(0)
            }
            "shift" => Some(0),
            "return" => Some(args.first().and_then(|s| s.parse().ok()).unwrap_or(0)),
            "break" => Some(0),
            "continue" => Some(0),
            "source" | "." => {
                if let Some(f) = args.first() {
                    if let Ok(c) = fs::read_to_string(f) {
                        return Some(self.run(&c));
                    }
                }
                Some(1)
            }
            "printf" => {
                let fmt = args.first().map(|s| s.as_str()).unwrap_or("%s\n");
                let rest = &args[1..];
                print!("{}", Self::fmt_str(fmt, rest));
                Some(0)
            }
            "test" | "[" => {
                let a = if args.last().map(|s| s.as_str()) == Some("]") {
                    &args[..args.len() - 1]
                } else {
                    args
                };
                Some(self.test(a))
            }
            "ulimit" | "shopt" | "trap" | "history" | "fc" => Some(0),
            _ => {
                if let Some(repl) = self.aliases.get(cmd) {
                    let mut full = vec![repl.clone()];
                    full.extend(args.to_vec());
                    return Some(self.run(&full.join(" ")));
                }
                None
            }
        }
    }

    fn fmt_str(fmt: &str, args: &[String]) -> String {
        let mut r = String::new();
        let cs: Vec<char> = fmt.chars().collect();
        let mut i = 0;
        let mut arg_idx = 0;

        while i < cs.len() {
            if cs[i] == '%' && i + 1 < cs.len() {
                let n = cs[i + 1];
                if let Some(val) = args.get(arg_idx) {
                    match n {
                        's' => r.push_str(val),
                        'd' | 'i' => r.push_str(&val.parse::<i64>().unwrap_or(0).to_string()),
                        'x' => r.push_str(&format!("{:x}", val.parse::<usize>().unwrap_or(0))),
                        '%' => r.push('%'),
                        _ => r.push_str(val),
                    }
                    arg_idx += 1;
                }
                i += 2;
            } else {
                r.push(cs[i]);
                i += 1;
            }
        }
        r
    }

    fn test(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            return 1;
        }
        if args.len() == 1 {
            return if args[0].is_empty() { 1 } else { 0 };
        }
        if args.len() == 2 {
            return match args[0].as_str() {
                "-z" => if args[1].is_empty() { 0 } else { 1 },
                "-n" => if args[1].is_empty() { 1 } else { 0 },
                "-f" => if Path::new(&args[1]).is_file() { 0 } else { 1 },
                "-d" => if Path::new(&args[1]).is_dir() { 0 } else { 1 },
                "-e" => if Path::new(&args[1]).exists() { 0 } else { 1 },
                _ => 1,
            };
        }
        if args.len() == 3 {
            let (a, op, b) = (&args[0], &args[1], &args[2]);
            return match op.as_str() {
                "=" | "==" => if a == b { 0 } else { 1 },
                "!=" => if a != b { 0 } else { 1 },
                "-eq" => if a.parse::<i64>().ok() == b.parse::<i64>().ok() { 0 } else { 1 },
                "-ne" => if a.parse::<i64>().ok() != b.parse::<i64>().ok() { 0 } else { 1 },
                "-lt" => if a.parse::<i64>().ok() < b.parse::<i64>().ok() { 0 } else { 1 },
                "-le" => if a.parse::<i64>().ok() <= b.parse::<i64>().ok() { 0 } else { 1 },
                "-gt" => if a.parse::<i64>().ok() > b.parse::<i64>().ok() { 0 } else { 1 },
                "-ge" => if a.parse::<i64>().ok() >= b.parse::<i64>().ok() { 0 } else { 1 },
                _ => 1,
            };
        }
        1
    }
}

fn which(name: &str) -> Option<String> {
    for dir in env::var("PATH").unwrap_or_default().split(':') {
        let p = Path::new(dir).join(name);
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "-c" {
        let cmd = if args.len() > 2 { args[2].clone() } else { String::new() };
        Shell::new().run(&cmd);
        return;
    }

    if args.len() > 1 {
        let content = fs::read_to_string(&args[1])
            .unwrap_or_else(|e| {
                eprintln!("bash: {}: {}", args[1], e);
                std::process::exit(127);
            });
        Shell::new().run(&content);
        return;
    }

    if !io::stdin().is_terminal() {
        let lines: Vec<String> = io::stdin().lines().filter_map(|l| l.ok()).collect();
        Shell::new().run(&lines.join("\n"));
        return;
    }

    let mut sh = Shell::new();
    let mut input = String::new();
    loop {
        print!("bash$ ");
        io::stdout().flush().ok();
        input.clear();
        if io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) {
            println!();
            break;
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }
        if line == "exit" {
            break;
        }
        sh.run(line);
    }
}