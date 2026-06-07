use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, exit};
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use std::os::unix::process::CommandExt;

// ─── Shell Context ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Redirect {
    fd: u32,
    op: String,
    target: String,
    heredoc_content: Option<String>,
    heredoc_literal: bool,
}

#[derive(Clone, Debug)]
enum ControlFlow {
    Break(usize),
    Continue(usize),
    Return(i32),
}

struct ShellContext {
    vars: HashMap<String, String>,
    exported: Vec<String>,
    readonly: Vec<String>,
    last_status: i32,
    last_bg_pid: Option<usize>,
    shell_pid: u32,
    positional: Vec<String>,
    funcs: HashMap<String, Vec<Vec<String>>>,
    loop_level: usize,
    dot_file: Option<PathBuf>,
    control_flow: Option<ControlFlow>,
    traps: HashMap<String, String>,
    func_depth: usize,
    shell_options: HashMap<String, bool>,
    pending_heredocs: HashMap<String, (String, bool)>,
}

impl ShellContext {
    fn new() -> Self {
        let pid = std::process::id();
        let mut ctx = Self {
            vars: HashMap::new(),
            exported: Vec::new(),
            readonly: Vec::new(),
            last_status: 0,
            last_bg_pid: None,
            shell_pid: pid,
            positional: Vec::new(),
            funcs: HashMap::new(),
            loop_level: 0,
            dot_file: None,
            control_flow: None,
            traps: HashMap::new(),
            func_depth: 0,
            shell_options: HashMap::new(),
            pending_heredocs: HashMap::new(),
        };
        ctx.vars.insert("$".to_string(), pid.to_string());
        ctx.vars.insert("?".to_string(), "0".to_string());
        for (key, value) in std::env::vars() {
            ctx.vars.insert(key, value);
        }
        ctx
    }

    fn get_var(&self, name: &str) -> String {
        if name == "?" {
            return self.last_status.to_string();
        }
        if name == "$" {
            return self.shell_pid.to_string();
        }
        if name == "!" {
            return self.last_bg_pid.map(|p| p.to_string()).unwrap_or_default();
        }
        if name == "#" {
            return self.positional.len().to_string();
        }
        if name == "@" || name == "*" {
            return if self.positional.is_empty() {
                String::new()
            } else if name == "*" {
                self.positional.join(" ")
            } else {
                self.positional.join(" ")
            };
        }
        // Positional params $1, $2, $3, ...
        if let Ok(n) = name.parse::<usize>() {
            if n > 0 && n <= self.positional.len() {
                return self.positional[n - 1].clone();
            }
        }
        if let Some(val) = self.vars.get(name) {
            return val.clone();
        }
        env::var(name).unwrap_or_default()
    }

    fn set_var(&mut self, name: &str, value: &str) {
        if self.readonly.contains(&name.to_string()) {
            eprintln!("sh: {}: is read only", name);
            return;
        }
        self.vars.insert(name.to_string(), value.to_string());
    }

    fn mark_exported(&mut self, name: &str) {
        if !self.exported.contains(&name.to_string()) {
            self.exported.push(name.to_string());
        }
    }

    fn push_vars_to_env(&self) {
        for name in &self.exported {
            if let Some(val) = self.vars.get(name) {
                unsafe { env::set_var(name, val); }
            }
        }
    }
}

// ─── Tokenizer ───────────────────────────────────────────────────────────────

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
            // In double quotes, \ only escapes \\ \" \$
            if in_dquote && c != '\\' && c != '"' && c != '$' {
                current.push('\\');
            }
            current.push(c);
            escape = false;
            i += 1;
            continue;
        }
        // Outside quotes: backslash escapes next char
        if c == '\\' && !in_quote {
            escape = true;
            i += 1;
            continue;
        }
        // Single quote
        if c == '\'' && !in_dquote {
            current.push(c);
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        // Double quote
        if c == '"' && !in_quote {
            current.push(c);
            in_dquote = !in_dquote;
            i += 1;
            continue;
        }
        // Whitespace delimiter
        if (c == ' ' || c == '\t') && !in_quote && !in_dquote {
            flush(&mut current, &mut tokens);
            i += 1;
            continue;
        }

        // Special chars outside quotes
        if !in_quote && !in_dquote {
            match c {
                '|' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == '|' {
                        tokens.push("||".to_string()); i += 2;
                    } else {
                        tokens.push("|".to_string()); i += 1;
                    }
                    continue;
                }
                ';' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == ';' {
                        if i + 2 < chars.len() && chars[i + 2] == '&' {
                            tokens.push(";;&".to_string()); i += 3;
                        } else {
                            tokens.push(";;".to_string()); i += 2;
                        }
                    } else {
                        tokens.push(";".to_string()); i += 1;
                    }
                    continue;
                }
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
                        if i + 2 < chars.len() && chars[i + 2] == '<' {
                            tokens.push("<<<".to_string()); i += 3;
                        } else {
                            tokens.push("<<".to_string()); i += 2;
                            // Parse heredoc delimiter preserving quotes
                            while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') { i += 1; }
                            if i < chars.len() && (chars[i] == '\'' || chars[i] == '"') {
                                // Quoted delimiter — preserve quotes
                                let q = chars[i];
                                let mut delim = String::new();
                                delim.push(q);
                                i += 1;
                                while i < chars.len() && chars[i] != q {
                                    match chars[i] {
                                        '\\' if q == '"' => {
                                            i += 1;
                                            if i < chars.len() { delim.push(chars[i]); i += 1; }
                                        }
                                        c => { delim.push(c); i += 1; }
                                    }
                                }
                                if i < chars.len() {
                                    delim.push(chars[i]); // closing quote
                                    i += 1;
                                }
                                tokens.push(delim);
                            }
                        }
                    } else if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push("<>".to_string()); i += 2;
                    } else {
                        tokens.push("<".to_string()); i += 1;
                    }
                    continue;
                }
                '>' => {
                    flush(&mut current, &mut tokens);
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(">>".to_string()); i += 2;
                    } else if i + 1 < chars.len() && chars[i + 1] == '&' {
                        tokens.push(">&".to_string()); i += 2;
                    } else {
                        tokens.push(">".to_string()); i += 1;
                    }
                    continue;
                }
                '$' => {
                    // Check for $(...) or $((...)) or ${...}
                    if i + 1 < chars.len() && chars[i+1] == '(' {
                        // If current ends with =, this is a variable assignment — keep as one token
                        if current.ends_with('=') {
                            current.push('$');
                            i += 1;
                            current.push(chars[i]); // push (
                            i += 1;
                            let mut depth = 1;
                            while i < chars.len() && depth > 0 {
                                if chars[i] == '(' { depth += 1; }
                                else if chars[i] == ')' { depth -= 1; }
                                if depth > 0 { current.push(chars[i]); i += 1; }
                            }
                            if i < chars.len() { current.push(')'); i += 1; }
                            continue;
                        }
                        flush(&mut current, &mut tokens);
                        i += 2; // skip $(
                        let mut depth = 1;
                        let start = i - 2;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '(' { depth += 1; }
                            else if chars[i] == ')' { depth -= 1; }
                            if depth > 0 { i += 1; }
                        }
                        if i < chars.len() { i += 1; } // skip closing )
                        tokens.push(chars[start..i].iter().collect());
                        continue;
                    }
                    if i + 1 < chars.len() && chars[i+1] == '{' {
                        if current.ends_with('=') {
                            current.push('$');
                            i += 1;
                            current.push(chars[i]); // push {
                            i += 1;
                            let mut depth = 1;
                            while i < chars.len() && depth > 0 {
                                if chars[i] == '{' { depth += 1; }
                                else if chars[i] == '}' { depth -= 1; }
                                if depth > 0 { current.push(chars[i]); i += 1; }
                            }
                            if i < chars.len() { current.push('}'); i += 1; }
                            continue;
                        }
                        flush(&mut current, &mut tokens);
                        i += 2; // skip ${
                        let mut depth = 1;
                        let start = i - 2;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' { depth += 1; }
                            else if chars[i] == '}' { depth -= 1; }
                            if depth > 0 { i += 1; }
                        }
                        if i < chars.len() { i += 1; } // skip }
                        tokens.push(chars[start..i].iter().collect());
                        continue;
                    }
                    // Regular $var — let it fall through to be accumulated
                    current.push('$');
                    i += 1;
                    continue;
                }
                '(' => {
                    // Standalone ( — flush and push as token
                    flush(&mut current, &mut tokens);
                    tokens.push("(".to_string());
                    i += 1;
                    continue;
                }
                ')' => {
                    flush(&mut current, &mut tokens);
                    tokens.push(")".to_string());
                    i += 1;
                    continue;
                }
                '{' | '}' => {
                    flush(&mut current, &mut tokens);
                    tokens.push(c.to_string());
                    i += 1;
                    continue;
                }
                _ => {}
            }
            // Handle 2>, 2>&1, 3>, etc.
            if c.is_ascii_digit() && i + 1 < chars.len() && chars[i + 1] == '>' {
                flush(&mut current, &mut tokens);
                if i + 2 < chars.len() && chars[i + 2] == '>' {
                    tokens.push(format!("{}>>", c));
                    i += 3;
                } else if i + 2 < chars.len() && chars[i + 2] == '&' {
                    if i + 3 < chars.len() && chars[i + 3] == '1' {
                        tokens.push(format!("{}>&1", c));
                        i += 4;
                    } else if i + 3 < chars.len() && chars[i + 3] == '-' {
                        tokens.push(format!("{}>&-", c));
                        i += 4;
                    } else {
                        tokens.push(format!("{}>&", c));
                        i += 3;
                    }
                } else {
                    tokens.push(format!("{}>", c));
                    i += 2;
                }
                continue;
            }
        }

        current.push(c);
        i += 1;
    }

    if in_quote || in_dquote {
        eprintln!("sh: syntax error: unclosed quote");
    }

    flush(&mut current, &mut tokens);
    tokens
}

// ─── Token classification ───────────────────────────────────────────────────

fn is_redirect_op(t: &str) -> bool {
    matches!(t, "<" | ">" | ">>" | "<<" | "<<<" | "<>" | ">&" | "2>" | "2>>" | "2>&1" | "2>&-")
        || (t.len() > 1 && t.ends_with('>') && t.chars().all(|c| c.is_ascii_digit() || c == '>'))
        || (t.len() > 2 && t.ends_with(">&") && t[..t.len()-2].chars().all(|c| c.is_ascii_digit()))
}

fn is_control_op(t: &str) -> bool {
    matches!(t, ";" | "&&" | "||" | "|" | "&")
}

// ─── Variable Expansion ──────────────────────────────────────────────────────

fn expand_vars(s: &str, ctx: &mut ShellContext) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '\'' {
            // Single-quoted string: literal, no expansion
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '\'' {
                out.push(chars[j]);
                j += 1;
            }
            i = j + 1;
            continue;
        }
        if c == '"' {
            // Double-quoted string: expand vars
            let mut j = i + 1;
            let mut inner = String::new();
            let mut cmdsub_depth = 0;
            while j < chars.len() {
                if cmdsub_depth == 0 && chars[j] == '"' {
                    break;
                }
                if chars[j] == '\\' && j + 1 < chars.len() && cmdsub_depth == 0 {
                    let next = chars[j + 1];
                    if next == '\\' || next == '"' || next == '$' {
                        inner.push(next);
                        j += 2;
                        continue;
                    }
                }
                if chars[j] == '$' && j + 1 < chars.len() && chars[j + 1] == '(' {
                    cmdsub_depth += 1;
                } else if chars[j] == ')' && cmdsub_depth > 0 {
                    cmdsub_depth -= 1;
                }
                inner.push(chars[j]);
                j += 1;
            }
            out.push_str(&expand_vars(&inner, ctx));
            i = j + 1;
            continue;
        }
        if c == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // ${var...}
                let mut j = i + 2;
                let mut var = String::new();
                let mut rest = String::new();
                let mut brace_depth = 1;
                while j < chars.len() && brace_depth > 0 {
                    match chars[j] {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                    if brace_depth > 0 {
                        rest.push(chars[j]);
                    }
                    j += 1;
                }
                // Parse var and optional operations
                let (varname, op, arg) = parse_var_op(&rest);
                let val = ctx.get_var(&varname);
                match op.as_str() {
                    ":-" => {
                        if val.is_empty() { out.push_str(&arg); }
                        else { out.push_str(&val); }
                    }
                    ":=" => {
                        if val.is_empty() { ctx.set_var(&varname, &arg); out.push_str(&arg); }
                        else { out.push_str(&val); }
                    }
                    ":+" => {
                        if !val.is_empty() { out.push_str(&arg); }
                    }
                    ":?" => {
                        if val.is_empty() {
                            eprintln!("sh: {}: {}", varname, arg);
                            // set error status
                        } else {
                            out.push_str(&val);
                        }
                    }
                    "#" => {
                        // ${#var} = length
                        out.push_str(&val.len().to_string());
                    }
                    "%" | "%%" | "#" | "##" => {
                        // Pattern removal - simplified
                        // ${var%pattern}, ${var%%pattern}, ${var#pattern}, ${var##pattern}
                        let p = &arg;
                        let stripped = match op.as_str() {
                            "#" => val.strip_prefix(p).unwrap_or(&val),
                            "##" => val.rsplit_once(p).map(|(_, s)| s).unwrap_or(&val),
                            "%" => val.strip_suffix(p).unwrap_or(&val),
                            "%%" => val.split_once(p).map(|(s, _)| s).unwrap_or(&val),
                            _ => &val,
                        };
                        out.push_str(stripped);
                    }
                    _ => {
                        out.push_str(&val);
                    }
                }
                i = j;
                continue;
            } else if chars[i + 1] == '(' && i + 2 < chars.len() && chars[i + 2] == '(' {
                // $(( ... )) arithmetic
                let mut j = i + 3;
                let mut depth = 2;
                while j < chars.len() && depth > 0 {
                    if chars[j] == '(' { depth += 1; }
                    if chars[j] == ')' { depth -= 1; }
                    if depth > 0 { j += 1; }
                }
                let expr: String = chars[i + 3..j - 1].iter().collect();
                let result = eval_arith(&expr, ctx);
                out.push_str(&result.to_string());
                i = j + 1;
                continue;
            } else if chars[i + 1] == '(' {
                // $( ... ) command substitution
                let mut j = i + 2;
                let mut depth = 1;
                while j < chars.len() && depth > 0 {
                    if chars[j] == '(' { depth += 1; }
                    if chars[j] == ')' { depth -= 1; }
                    j += 1;
                }
                let cmd: String = chars[i + 2..j - 1].iter().collect();
                let result = exec_cmd_subst(&cmd, ctx);
                out.push_str(&result);
                i = j;
                continue;
            } else {
                // Simple $var or $?
                let mut j = i + 1;
                let mut varname = String::new();
                if j < chars.len() && chars[j] == '?' {
                    varname.push('?');
                    j += 1;
                } else if j < chars.len() && chars[j] == '$' {
                    varname.push('$');
                    j += 1;
                } else if j < chars.len() && chars[j] == '!' {
                    varname.push('!');
                    j += 1;
                } else if j < chars.len() && chars[j] == '#' {
                    varname.push('#');
                    j += 1;
                } else if j < chars.len() && chars[j] == '@' {
                    varname.push('@');
                    j += 1;
                } else if j < chars.len() && chars[j] == '*' {
                    varname.push('*');
                    j += 1;
                } else if j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                        varname.push(chars[j]);
                        j += 1;
                    }
                } else {
                    // Standalone $, leave as is
                    out.push('$');
                    i = j;
                    continue;
                }
                let val = ctx.get_var(&varname);
                out.push_str(&val);
                i = j;
                continue;
            }
        }
        // Tilde expansion at word start
        if c == '~' && (out.is_empty() || out.ends_with(' ') || out.ends_with(':') || out.ends_with('=')) {
            let mut j = i + 1;
            let mut username = String::new();
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                username.push(chars[j]);
                j += 1;
            }
            let tilde_result = if username.is_empty() {
                env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else {
                format!("~{}", username)
            };
            out.push_str(&tilde_result);
            i = j;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn parse_var_op(s: &str) -> (String, String, String) {
    // ${var:-default}, ${var:=default}, ${var:+alt}, ${var:?err}, ${#var}, ${var%pat}
    let s = s.trim();
    if s.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    // Check for ${#var} pattern (length)
    if s.starts_with('#') {
        let varname = s[1..].to_string();
        return (varname, "#".to_string(), String::new());
    }
    // Check for :- := :+ :?
    for (i, c) in s.char_indices() {
        if c == ':' && i + 1 < s.len() {
            let next = &s[i..i+2];
            if matches!(next, ":-" | ":=" | ":+" | ":?") {
                let varname = s[..i].to_string();
                let arg = s[i+2..].to_string();
                return (varname, next.to_string(), arg);
            }
        }
        // Check for % %% # ## pattern removal
        if c == '%' || c == '#' {
            let op = if i + 1 < s.len() && s[i+1..].starts_with(c) {
                s[i..i+2].to_string()
            } else {
                c.to_string()
            };
            let varname = s[..i].to_string();
            let arg = s[i+op.len()..].to_string();
            return (varname, op, arg);
        }
    }
    (s.to_string(), String::new(), String::new())
}

// ─── Arithmetic Evaluation ───────────────────────────────────────────────────

fn eval_arith(expr: &str, ctx: &mut ShellContext) -> i64 {
    let expanded = expand_vars(expr, ctx);
    let e = expanded.trim();
    if e.is_empty() {
        return 0;
    }
    arith_parse_expr(e, ctx)
}

fn arith_parse_expr(s: &str, ctx: &ShellContext) -> i64 {
    let s = s.trim();
    // Handle + -
    let mut last_op = '+';
    let mut current = 0i64;
    let mut term_start = 0;
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => { depth += 1; }
            ')' => { depth -= 1; }
            '+' | '-' if depth == 0 && i > 0 => {
                let term = s[term_start..i].trim();
                let val = arith_parse_term(term, ctx);
                match last_op {
                    '+' => current += val,
                    '-' => current -= val,
                    _ => current = val,
                }
                last_op = c;
                term_start = i + 1;
            }
            _ => {}
        }
    }
    if term_start < s.len() {
        let term = s[term_start..].trim();
        let val = arith_parse_term(term, ctx);
        match last_op {
            '+' => current += val,
            '-' => current -= val,
            _ => current = val,
        }
    } else if last_op == '+' || last_op == '-' {
        // Handle leading sign
    }
    current
}

fn arith_parse_term(s: &str, ctx: &ShellContext) -> i64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    // Handle parenthesized sub-expression
    if s.starts_with('(') && s.ends_with(')') {
        return arith_parse_expr(&s[1..s.len()-1], ctx);
    }
    // Handle * / %
    let mut vals: Vec<(i64, char)> = Vec::new();
    let mut last_op = '*';
    let mut fact_start = 0;
    let mut depth = 0;
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '*' | '/' | '%' if depth == 0 => {
                let fact = s[fact_start..i].trim();
                let val = arith_parse_factor(fact, ctx);
                vals.push((val, last_op));
                last_op = c;
                fact_start = i + 1;
            }
            _ => {}
        }
    }
    if fact_start < s.len() {
        let fact = s[fact_start..].trim();
        let val = arith_parse_factor(fact, ctx);
        vals.push((val, last_op));
    }
    // Compute
    let mut result = 1i64;
    // Actually, compute sequentially
    if vals.is_empty() {
        return 0;
    }
    result = vals[0].0;
    for i in 1..vals.len() {
        match vals[i].1 {
            '*' => result *= vals[i].0,
            '/' => result /= vals[i].0,
            '%' => result %= vals[i].0,
            _ => result = vals[i].0,
        }
    }
    result
}

fn arith_parse_factor(s: &str, ctx: &ShellContext) -> i64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    // Handle parenthesized sub-expression
    if s.starts_with('(') && s.ends_with(')') {
        return arith_parse_expr(&s[1..s.len()-1], ctx);
    }
    // Handle unary + -
    if s.starts_with('-') {
        return -arith_parse_factor(&s[1..], ctx);
    }
    if s.starts_with('+') {
        return arith_parse_factor(&s[1..], ctx);
    }
    // Try to parse as number
    if let Ok(n) = s.parse::<i64>() {
        return n;
    }
    // Look up as shell variable
    if let Some(val) = ctx.vars.get(s) {
        if let Ok(n) = val.parse::<i64>() {
            return n;
        }
    }
    0
}

// ─── Command Substitution ────────────────────────────────────────────────────

fn exec_cmd_subst(cmd: &str, ctx: &ShellContext) -> String {
    // Spawn our own shell to handle command substitution
    // This ensures builtins (echo, test, etc.) work correctly
    let exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("sh"));
    let mut child = Command::new(exe.to_str().unwrap_or("sh"));
    child.arg("-c").arg(cmd);
    // Export shell variables as environment variables for the subshell
    for (k, v) in &ctx.vars {
        child.env(k, v);
    }
    child.stdout(Stdio::piped());
    child.stderr(Stdio::inherit());
    match child.output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim_end_matches('\n').to_string(),
        Err(_) => String::new(),
    }
}

// ─── Globbing ────────────────────────────────────────────────────────────────

fn glob_expand(pattern: &str) -> Vec<String> {
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return vec![pattern.to_string()];
    }
    // Simple glob: use std::fs
    let path = Path::new(pattern);
    let parent = path.parent().unwrap_or(Path::new("."));
    let file_pattern = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    let dir_str = parent.to_string_lossy();
    let dir = if dir_str.is_empty() { "." } else { &dir_str };

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![pattern.to_string()],
    };

    let mut results = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if glob_match(&name, &file_pattern) {
            let full = if dir == "." {
                name
            } else {
                format!("{}/{}", dir, name)
            };
            results.push(full);
        }
    }
    if results.is_empty() {
        vec![pattern.to_string()]
    } else {
        results.sort();
        results
    }
}

fn glob_match(name: &str, pattern: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    let mut pi = 0;
    let mut ni = 0;
    let mut backtrack_p = None;
    let mut backtrack_n = 0;

    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            backtrack_p = Some(pi);
            backtrack_n = ni;
            pi += 1;
        } else if let Some(bp) = backtrack_p {
            pi = bp + 1;
            backtrack_n += 1;
            ni = backtrack_n;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

// ─── Token expander ──────────────────────────────────────────────────────────

fn expand_tokens(tokens: &[String], ctx: &mut ShellContext) -> Vec<String> {
    let mut result = Vec::new();
    for t in tokens {
        if t.starts_with('$') && !t.starts_with("$(") {
            // Variable already partially expanded in tokenizer
            let expanded = expand_vars(t, ctx);
            // Glob expansion
            let globbed = glob_expand(&expanded);
            result.extend(globbed);
        } else if t.starts_with("$(") || t.starts_with("$") {
            let expanded = expand_vars(t, ctx);
            let globbed = glob_expand(&expanded);
            result.extend(globbed);
        } else if t.starts_with('~') && (t.len() == 1 || t.as_bytes().get(1).map_or(false, |c| *c == b'/')) {
            let expanded = expand_vars(t, ctx);
            let globbed = glob_expand(&expanded);
            result.extend(globbed);
        } else {
            result.push(t.clone());
        }
    }
    result
}

// ─── Command Execution ───────────────────────────────────────────────────────

fn execute_cmd(argv: &[String], redirects: &[Redirect], background: bool, ctx: &mut ShellContext) -> i32 {
    if argv.is_empty() {
        return 0;
    }

    let cmd = &argv[0];
    let cmd_args: Vec<String> = argv[1..].to_vec();

    // Check builtins
    if let Some(status) = exec_builtin(cmd, &cmd_args, redirects, ctx) {
        return status;
    }

    // External command
    let mut cmd_obj = Command::new(cmd);
    cmd_obj.args(&cmd_args);
    apply_redirects(&mut cmd_obj, redirects, ctx);
    ctx.push_vars_to_env();

    if background {
        match cmd_obj.spawn() {
            Ok(c) => {
                let pid = c.id();
                println!("[1] {}", pid);
                ctx.last_bg_pid = Some(pid as usize);
                0
            }
            Err(e) => {
                eprintln!("sh: {}: {}", cmd, e);
                127
            }
        }
    } else {
        match cmd_obj.status() {
            Ok(s) => s.code().unwrap_or(0),
            Err(e) => {
                eprintln!("sh: {}: {}", cmd, e);
                127
            }
        }
    }
}

// ─── Builtins ────────────────────────────────────────────────────────────────

fn exec_builtin(cmd: &str, args: &[String], redirects: &[Redirect], ctx: &mut ShellContext) -> Option<i32> {
    match cmd {
        "cd" => {
            let dir = if args.is_empty() || args[0] == "~" {
                env::var("HOME").unwrap_or_else(|_| "/".to_string())
            } else if args[0] == "-" {
                // cd - : go to previous directory
                let old = ctx.vars.get("OLDPWD").cloned().unwrap_or_else(|| "/".to_string());
                println!("{}", old);
                old
            } else {
                let expanded = expand_vars(&args[0], ctx);
                expanded
            };
            let prev = env::current_dir().ok().map(|p| p.to_string_lossy().to_string());
            if let Err(e) = env::set_current_dir(Path::new(&dir)) {
                eprintln!("cd: {}: {}", dir, e);
                Some(1)
            } else {
                if let Some(p) = prev {
                    ctx.vars.insert("OLDPWD".to_string(), p);
                }
                Some(0)
            }
        }
        "exit" => {
            let code = args.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(ctx.last_status);
            run_exit_traps(ctx);
            exit(code);
        }
        "export" => {
            if args.is_empty() {
                for name in &ctx.exported {
                    if let Some(val) = ctx.vars.get(name) {
                        println!("export {}='{}'", name, val);
                    }
                }
                return Some(0);
            }
            for arg in args {
                if let Some(eq) = arg.find('=') {
                    let name = &arg[..eq];
                    let val = &arg[eq+1..];
                    ctx.set_var(name, val);
                    ctx.mark_exported(name);
                    unsafe { env::set_var(name, val); }
                } else {
                    ctx.mark_exported(arg);
                }
            }
            Some(0)
        }
        "echo" => {
            let newline = !args.contains(&"-n".to_string());
            let mut i = 0;
            while i < args.len() && args[i] == "-n" {
                i += 1;
            }
            let out = args[i..].join(" ");
            let s = if newline { format!("{}\n", out) } else { out };
            #[cfg(unix)]
            {
                let mut written = 0;
                let buf = s.as_bytes();
                while written < buf.len() {
                    let ret = unsafe { libc::write(1, buf[written..].as_ptr() as *const libc::c_void, buf.len() - written) };
                    if ret < 0 {
                        let err = std::io::Error::last_os_error();
                        if err.kind() != std::io::ErrorKind::Interrupted {
                            break;
                        }
                    } else {
                        written += ret as usize;
                    }
                }
            }
            #[cfg(not(unix))]
            {
                if newline { println!("{}", s); } else { print!("{}", s); io::stdout().flush().ok(); }
            }
            Some(0)
        }
        "type" => {
            if args.is_empty() {
                eprintln!("type: usage: type name ...");
                return Some(1);
            }
            for arg in args {
                if is_builtin(arg) {
                    println!("{} is a shell builtin", arg);
                } else if ctx.funcs.contains_key(arg) {
                    println!("{} is a shell function", arg);
                } else if let Ok(path) = which_external(arg) {
                    println!("{} is {}", arg, path);
                } else {
                    println!("{}: not found", arg);
                }
            }
            Some(0)
        }
        "test" | "[" => {
            let test_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let result = exec_test(&test_args);
            Some(if result { 0 } else { 1 })
        }
        "eval" => {
            let cmd = args.join(" ");
            if !cmd.is_empty() {
                let status = exec_line(&cmd, ctx);
                Some(status)
            } else {
                Some(0)
            }
        }
        "." | "source" => {
            if args.is_empty() {
                eprintln!("sh: .: usage: . filename [arguments]");
                return Some(1);
            }
            let file = &args[0];
            // Search PATH if filename has no slash
            let resolved = if file.contains('/') {
                file.clone()
            } else {
                let path = env::var("PATH").unwrap_or_default();
                let mut found = String::new();
                for dir in path.split(':') {
                    let full = Path::new(dir).join(file);
                    if full.is_file() {
                        found = full.to_string_lossy().to_string();
                        break;
                    }
                }
                if found.is_empty() {
                    eprintln!("sh: .: {}: file not found", file);
                    return Some(1);
                }
                found
            };
            let prev_positional = ctx.positional.clone();
            ctx.positional = args[1..].to_vec();
            ctx.func_depth += 1;
            match fs::read_to_string(&resolved) {
                Ok(content) => {
                    let mut last_status = 0;
                    for line in content.lines() {
                        let trimmed = line.trim().to_string();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            continue;
                        }
                        last_status = exec_line(&trimmed, ctx);
                        if let Some(ControlFlow::Return(status)) = ctx.control_flow.take() {
                            last_status = status;
                            break;
                        }
                    }
                    ctx.func_depth -= 1;
                    ctx.positional = prev_positional;
                    Some(last_status)
                }
                Err(e) => {
                    ctx.func_depth -= 1;
                    eprintln!("sh: .: {}: {}", file, e);
                    ctx.positional = prev_positional;
                    Some(1)
                }
            }
        }
        "read" => {
            let mut i = 0;
            let mut raw = false;
            while i < args.len() && args[i].starts_with('-') {
                for c in args[i][1..].chars() {
                    match c { 'r' => raw = true, _ => {} }
                }
                i += 1;
            }
            let var = args.get(i).cloned().unwrap_or_else(|| "REPLY".to_string());
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) if n > 0 => {
                    let trimmed = if raw {
                        input.trim_end_matches('\n').trim_end_matches('\r').to_string()
                    } else {
                        // Handle backslash continuation
                        let s = input.trim_end_matches('\n').trim_end_matches('\r').to_string();
                        let mut result = String::new();
                        let mut chars = s.chars().peekable();
                        while let Some(c) = chars.next() {
                            if c == '\\' && chars.peek().is_some() {
                                // Skip backslash, next char is literal
                                result.push(chars.next().unwrap());
                            } else {
                                result.push(c);
                            }
                        }
                        result
                    };
                    ctx.set_var(&var, &trimmed);
                    Some(0)
                }
                _ => {
                    Some(1)
                }
            }
        }
        "exec" => {
            if args.is_empty() {
                return Some(0);
            }
            let mut cmd_obj = Command::new(&args[0]);
            cmd_obj.args(&args[1..]);
            apply_redirects(&mut cmd_obj, redirects, ctx);
            ctx.push_vars_to_env();
            #[cfg(unix)]
            {
                let err = cmd_obj.exec();
                eprintln!("sh: exec: {}: {}", args[0], err);
                Some(127)
            }
            #[cfg(not(unix))]
            {
                let status = cmd_obj.status().unwrap_or_else(|_| {
                    eprintln!("sh: exec: {}: command not found", args[0]);
                    std::process::exit(127);
                });
                std::process::exit(status.code().unwrap_or(0));
            }
        }
        "wait" => {
            #[cfg(unix)]
            {
                let ret = if let Some(pid_str) = args.first() {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        let mut status = 0;
                        unsafe { libc::waitpid(pid, &mut status, 0); }
                        if libc::WIFEXITED(status) {
                            libc::WEXITSTATUS(status)
                        } else {
                            0
                        }
                    } else {
                        eprintln!("sh: wait: {}: not a pid", pid_str);
                        return Some(1);
                    }
                } else {
                    // Wait for any child
                    let mut status = 0;
                    unsafe { libc::waitpid(-1, &mut status, 0); }
                    if libc::WIFEXITED(status) {
                        libc::WEXITSTATUS(status)
                    } else {
                        0
                    }
                };
                Some(ret)
            }
            #[cfg(not(unix))]
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
                Some(0)
            }
        }
        "readonly" => {
            if args.is_empty() {
                let mut keys: Vec<&String> = ctx.readonly.iter().collect();
                keys.sort();
                for k in keys {
                    if let Some(val) = ctx.vars.get(k) {
                        println!("readonly {}='{}'", k, val);
                    }
                }
                return Some(0);
            }
            for arg in args {
                if let Some(eq) = arg.find('=') {
                    let name = &arg[..eq];
                    let val = &arg[eq+1..];
                    ctx.set_var(name, val);
                    if !ctx.readonly.contains(&name.to_string()) {
                        ctx.readonly.push(name.to_string());
                    }
                } else {
                    if !ctx.readonly.contains(&arg.to_string()) {
                        ctx.readonly.push(arg.to_string());
                    }
                }
            }
            Some(0)
        }
"trap" => {
            if args.is_empty() {
                // List traps
                let mut sigs: Vec<String> = ctx.traps.keys().cloned().collect();
                sigs.sort();
                for sig in &sigs {
                    if let Some(cmd) = ctx.traps.get(sig) {
                        println!("trap -- '{}' {}", cmd, sig);
                    }
                }
                return Some(0);
            }
            if args.len() == 1 && args[0] == "-l" {
                // List signal names — simple implementation
                println!(" 1) SIGHUP       2) SIGINT       3) SIGQUIT      4) SIGILL       5) SIGTRAP");
                println!(" 6) SIGABRT      7) SIGEMT       8) SIGFPE       9) SIGKILL     10) SIGBUS");
                println!("11) SIGSEGV     12) SIGSYS      13) SIGPIPE     14) SIGALRM    15) SIGTERM");
                return Some(0);
            }
            let action = &args[0];
            let signals = &args[1..];
            if signals.is_empty() {
                // Single arg that is a signal name/number — reset it
                ctx.traps.remove(action);
                return Some(0);
            }
            for sig in signals {
                if action == "-" {
                    ctx.traps.remove(sig);
                } else {
                    ctx.traps.insert(sig.clone(), action.clone());
                }
            }
            Some(0)
        }
        "command" => {
            if args.is_empty() {
                eprintln!("sh: command: usage: command [-p] utility [argument ...]");
                return Some(1);
            }
            let mut skip = 0;
            if args[0] == "-p" { skip = 1; }
            if skip >= args.len() {
                return Some(0);
            }
            let cmd_name = &args[skip];
            let cmd_args: Vec<String> = args[skip + 1..].to_vec();
            // command: skip functions, only use builtins and external
            if let Some(status) = exec_builtin(cmd_name, &cmd_args, &[], ctx) {
                ctx.last_status = status;
                return Some(status);
            }
            // External command
            let mut cmd_obj = Command::new(cmd_name);
            cmd_obj.args(&cmd_args);
            ctx.push_vars_to_env();
            let status = cmd_obj.status().unwrap_or_else(|_| {
                eprintln!("sh: command: {}: not found", cmd_name);
                std::process::exit(127);
            });
            let code = status.code().unwrap_or(0);
            ctx.last_status = code;
            Some(code)
        }
        "shift" => {
            let n = args.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            if n > ctx.positional.len() {
                eprintln!("sh: shift: shift count out of range");
                return Some(1);
            }
            ctx.positional.drain(..n);
            ctx.vars.remove("OPTIND");
            ctx.vars.remove("_OPTPOS");
            Some(0)
        }
        "unset" => {
            for arg in args {
                ctx.vars.remove(arg);
                ctx.exported.retain(|e| e != arg);
            }
            Some(0)
        }
        "set" => {
            if args.is_empty() {
                // Print all shell variables
                let mut keys: Vec<&String> = ctx.vars.keys().collect();
                keys.sort();
                for k in keys {
                    println!("{}='{}'", k, ctx.vars.get(k).unwrap());
                }
                Some(0)
            } else if args[0] == "--" {
                // set -- ... : set positional parameters (skip --); reset OPTIND
                ctx.positional = args[1..].to_vec();
                ctx.vars.remove("OPTIND");
                ctx.vars.remove("_OPTPOS");
                Some(0)
            } else if args[0].starts_with('-') || args[0].starts_with('+') {
                // Handle options: set -e, set +e, set -o option, etc.
                let mut status = 0;
                let mut i = 0;
                while i < args.len() && (args[i].starts_with('-') || args[i].starts_with('+')) {
                    if args[i] == "--" {
                        i += 1;
                        break;
                    }
                    let enable = args[i].starts_with('-');
                    let chars: Vec<char> = if args[i].len() > 1 { args[i][1..].chars().collect() } else { vec![] };
                    if chars == ['o'] {
                        // set -o option or set +o option
                        i += 1;
                        if i < args.len() {
                            ctx.shell_options.insert(args[i].clone(), enable);
                        }
                    } else {
                        for c in chars {
                            match c {
                                'e' => { ctx.shell_options.insert("errexit".to_string(), enable); }
                                'u' => { ctx.shell_options.insert("nounset".to_string(), enable); }
                                'x' => { ctx.shell_options.insert("xtrace".to_string(), enable); }
                                'C' => { ctx.shell_options.insert("noclobber".to_string(), enable); }
                                'v' => { ctx.shell_options.insert("verbose".to_string(), enable); }
                                'n' => { ctx.shell_options.insert("noexec".to_string(), enable); }
                                'f' => { ctx.shell_options.insert("noglob".to_string(), enable); }
                                'm' => { ctx.shell_options.insert("monitor".to_string(), enable); }
                                _ => { eprintln!("sh: set: unknown option: -{}", c); status = 1; }
                            }
                        }
                    }
                    i += 1;
                }
                // Remaining args after options become positional params
                if i < args.len() {
                    let mut pos: Vec<String> = args[i..].to_vec();
                    if pos.first().map(|s| s.as_str()) == Some("--") {
                        pos.remove(0);
                    }
                    ctx.positional = pos;
                    ctx.vars.remove("OPTIND");
                    ctx.vars.remove("_OPTPOS");
                }
                Some(status)
            } else {
                // Set positional parameters
                let mut pos = args.to_vec();
                if pos.first().map(|s| s.as_str()) == Some("--") {
                    pos.remove(0);
                }
                ctx.positional = pos;
                ctx.vars.remove("OPTIND");
                ctx.vars.remove("_OPTPOS");
                Some(0)
            }
        }
        "return" => {
            let code = args.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(ctx.last_status);
            if ctx.func_depth > 0 {
                ctx.control_flow = Some(ControlFlow::Return(code));
                Some(code)
            } else {
                eprintln!("sh: return: can only return from a function or sourced script");
                Some(1)
            }
        }
        "break" => {
            let n = args.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            ctx.control_flow = Some(ControlFlow::Break(n));
            Some(0)
        }
        "continue" => {
            let n = args.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            ctx.control_flow = Some(ControlFlow::Continue(n));
            Some(0)
        }
        ":" => {
            // Null builtin — does nothing, returns 0
            Some(0)
        }
        "umask" => {
            #[cfg(unix)]
            {
                if args.is_empty() {
                    let mask = unsafe { libc::umask(0) };
                    unsafe { libc::umask(mask); }
                    println!("{:04o}", mask);
                    Some(0)
                } else {
                    let val = args[0].trim();
                    let mask_u32 = if val.starts_with('0') {
                        u32::from_str_radix(val, 8).unwrap_or(0o22)
                    } else {
                        val.parse::<u32>().unwrap_or(0o22)
                    };
                    let mask = mask_u32 as libc::mode_t;
                    unsafe { libc::umask(mask); }
                    Some(0)
                }
            }
            #[cfg(not(unix))]
            {
                eprintln!("sh: umask: not supported on this platform");
                Some(1)
            }
        }
        "getopts" => {
            if args.len() < 2 {
                eprintln!("sh: getopts: usage: getopts optstring var [args]");
                return Some(1);
            }
            let optstring = &args[0];
            let var_name = &args[1];
            let loop_args: Vec<String> = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                ctx.positional.clone()
            };
            let mut optind: usize = ctx.vars.get("OPTIND").and_then(|v| v.parse().ok()).unwrap_or(1);
            let optpos_key = "_OPTPOS".to_string();
            // _OPTPOS tracks which character in the current arg to process next (0-based)
            let mut optpos: usize = ctx.vars.get(&optpos_key).and_then(|v| v.parse().ok()).unwrap_or(0);
            loop {
                if optind < 1 || optind > loop_args.len() {
                    ctx.vars.remove(&optpos_key);
                    ctx.set_var(var_name, "?");
                    return Some(1);
                }
                let arg = &loop_args[optind - 1];
                if !arg.starts_with('-') || arg == "-" {
                    ctx.vars.remove(&optpos_key);
                    ctx.set_var(var_name, "?");
                    return Some(1);
                }
                if arg == "--" {
                    ctx.vars.remove(&optpos_key);
                    optind += 1;
                    ctx.set_var("OPTIND", &optind.to_string());
                    ctx.set_var(var_name, "?");
                    return Some(1);
                }
                let opt_chars: Vec<char> = arg[1..].chars().collect();
                if optpos >= opt_chars.len() {
                    // Move to next arg
                    optind += 1;
                    optpos = 0;
                    ctx.vars.remove(&optpos_key);
                    continue;
                }
                let opt_char = opt_chars[optpos];
                optpos += 1;
                if optpos < opt_chars.len() {
                    ctx.set_var(&optpos_key, &optpos.to_string());
                } else {
                    ctx.vars.remove(&optpos_key);
                    optind += 1;
                }
                // Check if this option is valid and takes an argument
                let opt_idx = optstring.find(opt_char);
                if opt_idx.is_none() {
                    if optstring.starts_with(':') {
                        ctx.set_var(var_name, "?");
                        ctx.set_var("OPTARG", &opt_char.to_string());
                    } else {
                        eprintln!("sh: getopts: illegal option -- {}", opt_char);
                        ctx.set_var(var_name, "?");
                        ctx.set_var("OPTARG", "");
                    }
                    ctx.set_var("OPTIND", &optind.to_string());
                    return Some(0);
                }
                let needs_arg = opt_idx.unwrap() + 1 < optstring.len()
                    && optstring.as_bytes()[opt_idx.unwrap() + 1] == b':';
                ctx.set_var(var_name, &opt_char.to_string());
if needs_arg {
                    if optpos < opt_chars.len() {
                        // Argument attached: -oarg
                        let rest: String = opt_chars[optpos..].iter().collect();
                        ctx.set_var("OPTARG", &rest);
                    } else if optind <= loop_args.len() {
                        // Next arg is the option argument
                        let next_arg = &loop_args[optind - 1];
                        ctx.set_var("OPTARG", next_arg);
                        optind += 1;
                    } else {
                        if optstring.starts_with(':') {
                            ctx.set_var(var_name, ":");
                            ctx.set_var("OPTARG", &opt_char.to_string());
                        } else {
                            eprintln!("sh: getopts: option requires an argument -- {}", opt_char);
                            ctx.set_var(var_name, "?");
                        }
                    }
                } else {
                    ctx.set_var("OPTARG", "");
                }
                ctx.set_var("OPTIND", &optind.to_string());
                return Some(0);
            }
        }
        "alias" => {
            // Simple alias support
            for arg in args {
                if let Some(eq) = arg.find('=') {
                    let name = &arg[..eq];
                    let val = &arg[eq+1..];
                    let val = val.trim_matches('\'');
                    ctx.vars.insert(format!("_alias_{}", name), val.to_string());
                }
            }
            Some(0)
        }
        "unalias" => {
            for arg in args {
                ctx.vars.remove(&format!("_alias_{}", arg));
            }
            Some(0)
        }
        _ => None,
    }
}

fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "cd" | "exit" | "export" | "echo" | "type" | "test" | "["
        | "eval" | "." | "source" | "read" | "exec" | "wait" | "shift"
        | "unset" | "set" | "return" | "break" | "continue" | "alias" | "unalias"
        | ":" | "readonly" | "trap" | "command" | "umask" | "getopts")
}

fn file_type_check_unix(path: &str, check: &str) -> bool {
    match check {
        "block" => fs::metadata(path).map(|m| m.file_type().is_block_device()).unwrap_or(false),
        "char" => fs::metadata(path).map(|m| m.file_type().is_char_device()).unwrap_or(false),
        "fifo" => fs::metadata(path).map(|m| m.file_type().is_fifo()).unwrap_or(false),
        "socket" => fs::metadata(path).map(|m| m.file_type().is_socket()).unwrap_or(false),
        _ => false,
    }
}

#[cfg(not(unix))]
trait FileTypeExt {
    fn is_block_device(&self) -> bool;
    fn is_char_device(&self) -> bool;
    fn is_fifo(&self) -> bool;
    fn is_socket(&self) -> bool;
}

#[cfg(not(unix))]
impl FileTypeExt for std::fs::FileType {
    fn is_block_device(&self) -> bool { false }
    fn is_char_device(&self) -> bool { false }
    fn is_fifo(&self) -> bool { false }
    fn is_socket(&self) -> bool { false }
}

fn which_external(name: &str) -> Result<String, ()> {
    let path = env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        let full = Path::new(dir).join(name);
        if full.is_file() {
            return Ok(full.to_string_lossy().to_string());
        }
    }
    Err(())
}

// ─── Test builtin ────────────────────────────────────────────────────────────

fn exec_test(args: &[&str]) -> bool {
    let argc = args.len();
    if args.last() == Some(&"]") {
        return exec_test(&args[..argc - 1]);
    }
    if argc == 0 {
        return false;
    }
    // Unary operators
    if argc == 2 {
        let op = args[0];
        let val = args[1];
        return match op {
            "-n" => !val.is_empty(),
            "-z" => val.is_empty(),
            "-d" => Path::new(val).is_dir(),
            "-f" => Path::new(val).is_file(),
            "-e" => Path::new(val).exists(),
            "-r" => fs::metadata(val).map(|m| m.permissions().readonly()).unwrap_or(false),
            "-w" => !fs::metadata(val).map(|m| m.permissions().readonly()).unwrap_or(true),
            "-x" => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::metadata(val).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
                }
                #[cfg(not(unix))]
                {
                    Path::new(val).is_file()
                }
            }
            "-s" => fs::metadata(val).map(|m| m.len() > 0).unwrap_or(false),
            "-L" | "-h" => fs::symlink_metadata(val).map(|m| m.file_type().is_symlink()).unwrap_or(false),
            "-b" => file_type_check_unix(val, "block"),
            "-c" => file_type_check_unix(val, "char"),
            "-p" => file_type_check_unix(val, "fifo"),
            "-S" => file_type_check_unix(val, "socket"),
            "-t" => {
                if val == "0" { io::stdin().is_terminal() }
                else if val == "1" { io::stdout().is_terminal() }
                else if val == "2" { io::stderr().is_terminal() }
                else { false }
            }
            "-u" => false, // setuid not supported in cross-platform
            "-g" => false, // setgid not supported
            "-k" => false,
            "-O" => false,
            "-G" => false,
            "-N" => false,
            _ => !val.is_empty(),
        };
    }
    // Binary operators
    if argc == 3 {
        let left = args[0];
        let op = args[1];
        let right = args[2];
        return match op {
            "=" | "==" => left == right,
            "!=" => left != right,
            "-eq" => left.parse::<i64>().unwrap_or(0) == right.parse::<i64>().unwrap_or(0),
            "-ne" => left.parse::<i64>().unwrap_or(0) != right.parse::<i64>().unwrap_or(0),
            "-lt" => left.parse::<i64>().unwrap_or(0) < right.parse::<i64>().unwrap_or(0),
            "-le" => left.parse::<i64>().unwrap_or(0) <= right.parse::<i64>().unwrap_or(0),
            "-gt" => left.parse::<i64>().unwrap_or(0) > right.parse::<i64>().unwrap_or(0),
            "-ge" => left.parse::<i64>().unwrap_or(0) >= right.parse::<i64>().unwrap_or(0),
            "-nt" => file_mtime(left) > file_mtime(right),
            "-ot" => file_mtime(left) < file_mtime(right),
            "-ef" => file_inode(left) == file_inode(right),
            _ => false,
        };
    }
    // Single argument
    if argc == 1 {
        return !args[0].is_empty();
    }
    false
}

fn file_mtime(path: &str) -> u64 {
    fs::metadata(path).map(|m| {
        m.modified().map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()).unwrap_or(0)
    }).unwrap_or(0)
}

fn file_inode(path: &str) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(path).map(|m| m.ino()).unwrap_or(0)
    }
    #[cfg(not(unix))]
    {
        file_mtime(path)
    }
}

// ─── Parse redirects ─────────────────────────────────────────────────────────

fn parse_redirects(tokens: &[String], ctx: &mut ShellContext, mut line_idx: usize, input_lines: &[String]) -> (Vec<Redirect>, usize) {
    let mut redirects = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let (fd, op) = match tokens[i].as_str() {
            "<" => (0u32, "<"),
            ">" => (1u32, ">"),
            ">>" => (1u32, ">>"),
            "<<" => (0u32, "<<"),
            "<<<" => (0u32, "<<<"),
            "<>" => (0u32, "<>"),
            ">&" => (1u32, ">&"),
            "2>" => (2u32, ">"),
            "2>>" => (2u32, ">>"),
            "2>&1" => (2u32, ">&1"),
            "2>&-" => (2u32, ">&-"),
            _ => {
                // Check for numbered redirect like 3>, 4>>
                let s = tokens[i].as_str();
                if let Some(n) = s.strip_suffix('>') {
                    if n.chars().all(|c| c.is_ascii_digit()) {
                        let fd: u32 = n.parse().unwrap_or(1);
                        (fd, ">")
                    } else if let Some(n) = s.strip_suffix(">>") {
                        if n.chars().all(|c| c.is_ascii_digit()) {
                            let fd: u32 = n.parse().unwrap_or(1);
                            (fd, ">>")
                        } else {
                            i += 1; continue;
                        }
                    } else {
                        i += 1; continue;
                    }
                } else {
                    i += 1; continue;
                }
            }
        };

        match op {
            "<<" => {
                // Heredoc
                if i + 1 < tokens.len() {
                    let mut delim = tokens[i + 1].clone();
                    let is_literal = delim.starts_with('\'') || delim.starts_with('"');
                    if is_literal {
                        delim = delim[1..delim.len()-1].to_string();
                    }
                    // First check pending_heredocs (from collect_complete_lines)
                    let content = if let Some((mut c, lit)) = ctx.pending_heredocs.remove(&delim) {
                        if !lit {
                            c = expand_vars(&c, ctx);
                        }
                        c
                    } else {
                        // Fallback: read from input_lines
                        let mut c = String::new();
                        while line_idx < input_lines.len() {
                            let l = input_lines[line_idx].trim_end().to_string();
                            line_idx += 1;
                            if l.trim() == delim { break; }
                            c.push_str(&l);
                            c.push('\n');
                        }
                        c
                    };
                    redirects.push(Redirect {
                        fd: 0, op: "<<".to_string(), target: delim,
                        heredoc_content: Some(content),
                        heredoc_literal: is_literal,
                    });
                }
                i += 2;
                continue;
            }
            "<<<" => {
                // Here-string
                if i + 1 < tokens.len() {
                    let val = tokens[i + 1].clone();
                    redirects.push(Redirect {
                        fd: 0, op: "<<<".to_string(), target: String::new(),
                        heredoc_content: Some(val),
                        heredoc_literal: false,
                    });
                }
                i += 2;
                continue;
            }
            _ => {}
        }

        if i + 1 < tokens.len() {
            let target = tokens[i + 1].clone();
            redirects.push(Redirect {
                fd, op: op.to_string(), target, heredoc_content: None,
                heredoc_literal: false,
            });
        }
        i += 2;
    }

    (redirects, line_idx)
}

// ─── Apply redirects ─────────────────────────────────────────────────────────

fn apply_redirects(cmd: &mut Command, redirects: &[Redirect], _ctx: &mut ShellContext) {
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
                    match r.fd {
                        1 => { cmd.stdout(f); }
                        2 => { cmd.stderr(f); }
                        _ => {}
                    }
                }
            }
            ">&1" => {
                if r.fd == 2 {
                    cmd.stderr(Stdio::inherit());
                }
            }
            ">&-" => {
                // Close fd
                match r.fd {
                    1 => { cmd.stdout(Stdio::null()); }
                    2 => { cmd.stderr(Stdio::null()); }
                    _ => {}
                }
            }
            ">&" => {
                // Merge stderr->stdout or similar
                match r.fd {
                    2 => { cmd.stderr(Stdio::inherit()); }
                    _ => {}
                }
                if r.target == "1" && r.fd == 2 {
                    cmd.stderr(Stdio::inherit());
                }
            }
            "<>" => {
                if let Ok(f) = fs::OpenOptions::new().read(true).write(true).create(true).open(&r.target) {
                    cmd.stdin(f);
                }
            }
            "<<" | "<<<" => {
                if let Some(ref content) = r.heredoc_content {
                    let temp = format!("/tmp/sh_heredoc_{}_{}", std::process::id(),
                        std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_nanos());
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

// ─── Pipeline execution ──────────────────────────────────────────────────────

// ─── Apply redirects for builtins (fd-level) ─────────────────────────────────
// Two platform-specific implementations (unix vs riscv64), mutually exclusive cfgs.

#[cfg(unix)]
fn apply_redirects_builtin(redirects: &[Redirect]) -> Vec<(i32, i32)> {
    use std::os::unix::io::IntoRawFd;
    let mut saved: Vec<(i32, i32)> = Vec::new();
    for r in redirects {
        match r.op.as_str() {
            ">" | ">>" => {
                if r.fd != 1 && r.fd != 2 { continue; }
                let f = if r.op == ">>" {
                    fs::OpenOptions::new().create(true).append(true).open(&r.target)
                } else {
                    fs::File::create(&r.target)
                };
                    if let Ok(file) = f {
                        let target_fd = r.fd as i32;
                        let new_fd = file.into_raw_fd();
                        let old = unsafe { libc::dup(target_fd) };
                        unsafe { libc::dup2(new_fd, target_fd); }
                        unsafe { libc::close(new_fd); }
                        saved.push((target_fd, old));
                    } else {
                        eprintln!("sh: cannot create '{}'", r.target);
                    }
            }
            "<" => {
                if r.fd != 0 { continue; }
                if let Ok(file) = fs::File::open(&r.target) {
                    let target_fd = r.fd as i32;
                    let new_fd = file.into_raw_fd();
                    let old = unsafe { libc::dup(target_fd) };
                    unsafe { libc::dup2(new_fd, target_fd); }
                    unsafe { libc::close(new_fd); }
                    saved.push((target_fd, old));
                }
            }
            _ => {}
        }
    }
    saved
}

#[cfg(unix)]
fn restore_fds(saved: Vec<(i32, i32)>) {
    for (fd, old_fd) in saved {
        if old_fd >= 0 {
            unsafe { libc::dup2(old_fd, fd); }
            unsafe { libc::close(old_fd); }
        }
    }
}

#[cfg(target_arch = "riscv64")]
fn apply_redirects_builtin(redirects: &[Redirect]) -> Vec<(i32, i32)> {
    use xv8_libc::{OpenFlag, dup, dup2, close, open};
    use std::ffi::CString;
    let mut saved: Vec<(i32, i32)> = Vec::new();
    for r in redirects {
        match r.op.as_str() {
            ">" | ">>" => {
                if r.fd != 1 && r.fd != 2 { continue; }
                let flags = if r.op == ">>" {
                    OpenFlag::WRITE_ONLY | OpenFlag::CREATE | OpenFlag::APPEND
                } else {
                    OpenFlag::WRITE_ONLY | OpenFlag::CREATE | OpenFlag::TRUNCATE
                };
                let cpath = CString::new(r.target.as_str()).unwrap();
                let new_fd = unsafe { open(cpath.as_ptr() as *const u8, flags) };
                if new_fd >= 0 {
                    let target_fd = r.fd as i32;
                    let old = unsafe { dup(target_fd as usize) } as i32;
                    unsafe { dup2(new_fd as usize, target_fd as usize); }
                    unsafe { close(new_fd as usize); }
                    saved.push((target_fd, old));
                } else {
                    eprintln!("sh: cannot create '{}'", r.target);
                }
            }
            "<" => {
                if r.fd != 0 { continue; }
                let cpath = CString::new(r.target.as_str()).unwrap();
                let new_fd = unsafe { open(cpath.as_ptr() as *const u8, OpenFlag::READ_ONLY) };
                if new_fd >= 0 {
                    let target_fd = r.fd as i32;
                    let old = unsafe { dup(target_fd as usize) } as i32;
                    unsafe { dup2(new_fd as usize, target_fd as usize); }
                    unsafe { close(new_fd as usize); }
                    saved.push((target_fd, old));
                }
            }
            _ => {}
        }
    }
    saved
}

#[cfg(target_arch = "riscv64")]
fn restore_fds(saved: Vec<(i32, i32)>) {
    use xv8_libc::{dup2, close};
    for (fd, old_fd) in saved {
        if old_fd >= 0 {
            unsafe { dup2(old_fd as usize, fd as usize); }
            unsafe { close(old_fd as usize); }
        }
    }
}

fn exec_pipeline(segments: &[Vec<String>], redirects: &[Redirect], background: bool, ctx: &mut ShellContext) -> i32 {
    if segments.is_empty() {
        return 0;
    }
    let mut children: Vec<std::process::Child> = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;

    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() {
            continue;
        }
        let (seg_redirects, _) = parse_redirects(seg, ctx, 0, &[]);
        let argv: Vec<String> = seg.iter()
            .filter(|t| !is_redirect_op(t) && !is_redirect_target(seg, t))
            .cloned()
            .collect();

        if argv.is_empty() {
            continue;
        }

        if is_builtin(&argv[0]) {
            // Builtins in pipeline: run in a subprocess
            let mut cmd = Command::new(if cfg!(target_os = "linux") { "sh" } else { "sh" });
            cmd.arg("-c").arg(&format!("{} {}", argv[0], argv[1..].join(" ")));

            if let Some(prev) = prev_stdout.take() {
                cmd.stdin(prev);
            }
            apply_redirects(&mut cmd, &seg_redirects, ctx);
            if i == segments.len() - 1 {
                apply_redirects(&mut cmd, redirects, ctx);
            }
            if i < segments.len() - 1 {
                cmd.stdout(Stdio::piped());
            }
            if let Ok(mut c) = cmd.spawn() {
                prev_stdout = c.stdout.take();
                children.push(c);
            }
            continue;
        }

        let mut cmd = Command::new(&argv[0]);
        cmd.args(&argv[1..]);

        if let Some(prev) = prev_stdout.take() {
            cmd.stdin(prev);
        }
        apply_redirects(&mut cmd, &seg_redirects, ctx);
        if i == segments.len() - 1 {
            apply_redirects(&mut cmd, redirects, ctx);
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
                for mut c in children {
                    let _ = c.wait();
                }
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

fn is_redirect_target(tokens: &[String], t: &str) -> bool {
    for i in 0..tokens.len() {
        if tokens[i] == t && i > 0 && is_redirect_op(&tokens[i - 1]) {
            return true;
        }
    }
    false
}

// ─── AST and Control Flow ──────────────────────────────────────────────────

#[derive(Debug)]
struct CmdNode {
    tokens: Vec<String>,    // after expansion
    redirects: Vec<Redirect>,
    background: bool,
}

#[derive(Debug)]
enum PipelineOp {
    Seq,     // ;
    And,
    Or,
    Pipe,    // |
}

#[derive(Debug)]
enum Ast {
    Cmd(CmdNode),
    Pipeline(Vec<Ast>, bool),  // segments, negated (!)
    Sequence(Box<Ast>, PipelineOp, Box<Ast>),
    Background(Box<Ast>),
    If {
        cond: Box<Ast>,
        then: Box<Ast>,
        else_: Option<Box<Ast>>,
    },
    For {
        var: String,
        words: Vec<String>,
        body: Box<Ast>,
    },
    While {
        cond: Box<Ast>,
        body: Box<Ast>,
        until: bool,
    },
    Case {
        word: String,
        patterns: Vec<(Vec<String>, Box<Ast>)>,
    },
    Function {
        name: String,
        body: Box<Ast>,
    },
    Subshell(Box<Ast>),
    BraceGroup(Box<Ast>),
    Break(Option<usize>),
    Continue(Option<usize>),
    Return(Option<i32>),
}

// ─── Line continuation helper ────────────────────────────────────────────────

fn collect_complete_lines(input: &str, ctx: &mut ShellContext) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut heredoc_body = String::new();
    let mut heredoc_delim: Option<(String, bool)> = None;
    let mut heredoc_just_closed = false;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut kw_depth: i32 = 0;
    let mut in_sq = false;
    let mut in_dq = false;

    for line in input.lines() {
        let trimmed = line.trim_end();

        if let Some((ref delim, ref is_lit)) = heredoc_delim {
            let trimmed_line = trimmed.trim().to_string();
            if trimmed_line == *delim {
                let d = delim.clone();
                let body = heredoc_body.clone();
                let lit = *is_lit;
                heredoc_delim = None;
                heredoc_just_closed = true;
                ctx.pending_heredocs.insert(d, (body, lit));
                continue;
            }
            if !heredoc_body.is_empty() {
                heredoc_body.push('\n');
            }
            heredoc_body.push_str(trimmed);
            continue;
        }

        if trimmed.ends_with('\\') {
            current.push_str(&trimmed[..trimmed.len()-1]);
            current.push('\n');
            continue;
        }

        for c in trimmed.chars() {
            if c == '\'' && !in_dq { in_sq = !in_sq; }
            else if c == '"' && !in_sq { in_dq = !in_dq; }
            else if !in_sq && !in_dq {
                if c == '{' { brace_depth += 1; }
                else if c == '}' { brace_depth -= 1; }
                else if c == '(' { paren_depth += 1; }
                else if c == ')' { paren_depth -= 1; }
            }
        }

        let tokens = tokenize(trimmed);
        for tok in &tokens {
            match tok.as_str() {
                "if" | "for" | "while" | "until" | "case" => kw_depth += 1,
                "fi" | "done" | "esac" => kw_depth -= 1,
                _ => {}
            }
        }

        if !current.is_empty() && !heredoc_just_closed {
            current.push_str("; ");
        }
        current.push_str(trimmed);

        if heredoc_just_closed && heredoc_delim.is_none() {
            if brace_depth <= 0 && paren_depth <= 0 && kw_depth <= 0 {
                lines.push(current.clone());
                current.clear();
                heredoc_just_closed = false;
                if brace_depth < 0 { brace_depth = 0; }
                if paren_depth < 0 { paren_depth = 0; }
                if kw_depth < 0 { kw_depth = 0; }
                continue;
            }
            heredoc_just_closed = false;
        }

        if brace_depth <= 0 && paren_depth <= 0 && kw_depth <= 0 {
            let ctokens = tokenize(&current);
            let mut has_heredoc = false;
            for i in 0..ctokens.len() {
                if ctokens[i] == "<<" && i + 1 < ctokens.len() {
                    let raw_delim = &ctokens[i + 1];
                    let lit = raw_delim.starts_with('\'') || raw_delim.starts_with('"');
                    let delim = if lit { raw_delim[1..raw_delim.len()-1].to_string() } else { raw_delim.clone() };
                    heredoc_delim = Some((delim, lit));
                    heredoc_body.clear();
                    has_heredoc = true;
                    break;
                }
            }
            if !has_heredoc {
                lines.push(current.clone());
                current.clear();
            }
            if brace_depth < 0 { brace_depth = 0; }
            if paren_depth < 0 { paren_depth = 0; }
            if kw_depth < 0 { kw_depth = 0; }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// ─── Parse one command from tokens ──────────────────────────────────────────

fn parse_command(tokens: &[String]) -> CmdNode {
    let mut filtered = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "|" {
            break;
        }
        // Skip control ops at top level
        if tokens[i] == ";" || tokens[i] == "&&" || tokens[i] == "||" || tokens[i] == "&" {
            break;
        }
        filtered.push(tokens[i].clone());
        i += 1;
    }
    let background = tokens.last().map(|t| t == "&").unwrap_or(false);
    let cmd_tokens: Vec<String> = if background {
        filtered.iter().filter(|t| *t != "&").cloned().collect()
    } else {
        filtered
    };
    // Parse redirects from cmd_tokens
    let mut argv = Vec::new();
    let mut redirects = Vec::new();
    let mut j = 0;
    while j < cmd_tokens.len() {
        if is_redirect_op(&cmd_tokens[j]) {
            let op = cmd_tokens[j].clone();
            let fd = if op.starts_with('2') { 2u32 } else { 0u32 };
            let target = if j + 1 < cmd_tokens.len() { cmd_tokens[j+1].clone() } else { String::new() };
            redirects.push(Redirect {
                fd,
                op,
                target,
                heredoc_content: None,
                heredoc_literal: false,
            });
            j += 2;
        } else if j + 1 < cmd_tokens.len() && is_redirect_op(&cmd_tokens[j+1]) {
            // fd + op
            j += 1;
        } else {
            argv.push(cmd_tokens[j].clone());
            j += 1;
        }
    }
    CmdNode { tokens: argv, redirects, background }
}

// ─── Execute AST ─────────────────────────────────────────────────────────────

fn exec_ast(ast: &Ast, ctx: &mut ShellContext) -> i32 {
    match ast {
        Ast::Cmd(node) => {
            let expanded: Vec<String> = node.tokens.iter()
                .flat_map(|t| {
                    let expanded = expand_vars(t, ctx);
                    glob_expand(&expanded)
                })
                .collect();
            if expanded.is_empty() {
                return 0;
            }
            execute_cmd(&expanded, &node.redirects, node.background, ctx)
        }
        Ast::Pipeline(segments, negated) => {
            let seg_tokens: Vec<Vec<String>> = segments.iter().map(|s| {
                if let Ast::Cmd(n) = s {
                    n.tokens.iter()
                        .flat_map(|t| {
                            let expanded = expand_vars(t, ctx);
                            glob_expand(&expanded)
                        })
                        .collect()
                } else {
                    vec![]
                }
            }).collect();
            let status = exec_pipeline(&seg_tokens, &[], false, ctx);
            if *negated { if status == 0 { 1 } else { 0 } } else { status }
        }
        Ast::Sequence(left, op, right) => {
            let left_status = exec_ast(left, ctx);
            let should_run = match op {
                PipelineOp::Seq => true,
                PipelineOp::And => left_status == 0,
                PipelineOp::Or => left_status != 0,
                PipelineOp::Pipe => true, // handled differently
            };
            if should_run {
                exec_ast(right, ctx)
            } else {
                left_status
            }
        }
        Ast::Background(inner) => {
            // Fork and execute in background
            let status = exec_ast(inner, ctx);
            println!("[1] done (status {})", status);
            0
        }
        Ast::If { cond, then, else_ } => {
            let cond_status = exec_ast(cond, ctx);
            if cond_status == 0 {
                exec_ast(then, ctx)
            } else if let Some(el) = else_ {
                exec_ast(el, ctx)
            } else {
                0
            }
        }
        Ast::For { var, words, body } => {
            let mut last_status = 0;
            for word in words {
                ctx.set_var(var, word);
                last_status = exec_ast(body, ctx);
            }
            last_status
        }
        Ast::While { cond, body, until } => {
            let mut last_status = 0;
            while {
                let s = exec_ast(cond, ctx);
                if *until { s != 0 } else { s == 0 }
            } {
                ctx.loop_level += 1;
                last_status = exec_ast(body, ctx);
                ctx.loop_level -= 1;
            }
            last_status
        }
        Ast::Case { word, patterns } => {
            let w = expand_vars(word, ctx);
            for (pattern, body) in patterns {
                for p in pattern {
                    let expanded = expand_vars(p, ctx);
                    if glob_match(&w, &expanded) {
                        return exec_ast(body, ctx);
                    }
                }
            }
            0
        }
        Ast::Function { name, body } => {
            // Store function body as tokens
            if let Ast::Cmd(node) = body.as_ref() {
                ctx.funcs.insert(name.clone(), vec![node.tokens.clone()]);
            }
            0
        }
        Ast::Subshell(inner) => {
            let tokens = flatten_ast(inner);
            let cmd = tokens.join(" ");
            let output = Command::new("sh").arg("-c").arg(&cmd).output();
            match output {
                Ok(o) => o.status.code().unwrap_or(0),
                Err(_) => exec_ast(inner, ctx),
            }
        }
        Ast::BraceGroup(inner) => {
            exec_ast(inner, ctx)
        }
        Ast::Break(_n) => {
            if ctx.loop_level > 0 {
                // Signal break
                0
            } else {
                eprintln!("sh: break: outside loop");
                1
            }
        }
        Ast::Continue(_n) => {
            if ctx.loop_level > 0 {
                0
            } else {
                eprintln!("sh: continue: outside loop");
                1
            }
        }
        Ast::Return(code) => {
            code.unwrap_or(ctx.last_status)
        }
    }
}

fn flatten_ast(ast: &Ast) -> Vec<String> {
    match ast {
        Ast::Cmd(node) => node.tokens.clone(),
        _ => vec![],
    }
}

// ─── Token-based parsing for control flow ────────────────────────────────────

fn parse_and_exec(tokens: &[String], ctx: &mut ShellContext) -> i32 {
    if tokens.is_empty() {
        return 0;
    }

    // Check for function definition: name() { body }
    if tokens.len() >= 4 && tokens[1] == "(" && tokens[2] == ")" && tokens[3] == "{" {
        let name = tokens[0].clone();
        let close = tokens.iter().rposition(|t| t == "}");
        if let Some(cp) = close {
            if cp >= 4 {
                let body_tokens = &tokens[4..cp];
                ctx.funcs.insert(name, vec![body_tokens.to_vec()]);
                // Process tokens after the closing brace (e.g., `}; foo`)
                if cp + 1 < tokens.len() {
                    return exec_group_tokens(&tokens[cp + 1..], ctx);
                }
                return 0;
            }
        }
        eprintln!("sh: syntax error: unclosed function body");
        return 1;
    }

    // Check for control flow keywords at start
    let first = tokens[0].as_str();

    match first {
        "if" | "elif" => {
            // Parse if/then/elif/else/fi
            let (cond_tokens, after_cond) = split_after_token(tokens, "then");
            if after_cond.is_empty() {
                eprintln!("sh: syntax error: expected 'then' after 'if'");
                return 1;
            }
            let cond = parse_and_return_tokens(&cond_tokens[1..]); // skip if
            let (then_tokens, rest) = split_after_token_multi(&after_cond, &["else", "elif", "fi"]);
            let mut else_ast = None;

            if rest.first().map(|s| s.as_str()) == Some("else") {
                let (else_tokens, _) = split_after_token(&rest[1..], "fi");
                else_ast = Some(else_tokens);
            } else if rest.first().map(|s| s.as_str()) == Some("elif") {
                // Nested elif → recursive if (full rest passed to parse_and_exec)
                else_ast = Some(rest.clone());
            }

            let cond_status = exec_group_tokens(&cond, ctx);
            if cond_status == 0 {
                exec_group_tokens(&then_tokens, ctx)
            } else if let Some(el) = else_ast {
                if rest.first().map(|s| s.as_str()) == Some("elif") {
                    parse_and_exec(&el, ctx)
                } else {
                    exec_group_tokens(&el, ctx)
                }
            } else {
                0
            }
        }
        "for" => {
            if tokens.len() < 4 || tokens[2] != "in" {
                eprintln!("sh: syntax error: for var in words ...");
                return 1;
            }
            let var = tokens[1].clone();
            // Find "do"
            let do_pos = tokens.iter().position(|t| t == "do").unwrap_or(tokens.len());
            let words: Vec<String> = tokens[3..do_pos].iter()
                .filter(|t| *t != ";" && *t != "in")
                .cloned()
                .collect();
            let (body, _) = split_after_token(&tokens[do_pos + 1..], "done");
            let mut last_status = 0;
            for word in &words {
                ctx.set_var(&var, word);
                last_status = exec_group_tokens(&body, ctx);
                if let Some(sig) = ctx.control_flow.take() {
                    match sig {
                        ControlFlow::Break(_) => { break; }
                        ControlFlow::Continue(_) => { continue; }
                        ControlFlow::Return(_) => { ctx.control_flow = Some(sig); break; }
                    }
                }
            }
            last_status
        }
        "while" | "until" => {
            let until = first == "until";
            let do_pos = tokens.iter().position(|t| t == "do").unwrap_or(tokens.len());
            let cond_tokens = parse_and_return_tokens(&tokens[1..do_pos]);
            let (body, _) = split_after_token(&tokens[do_pos + 1..], "done");
            let mut last_status = 0;
            let mut cond_status = exec_group_tokens(&cond_tokens, ctx);
            while if until { cond_status != 0 } else { cond_status == 0 } {
                ctx.loop_level += 1;
                last_status = exec_group_tokens(&body, ctx);
                if let Some(sig) = ctx.control_flow.take() {
                    match sig {
                        ControlFlow::Break(_) => { ctx.loop_level -= 1; break; }
                        ControlFlow::Continue(_) => { ctx.loop_level -= 1; cond_status = exec_group_tokens(&cond_tokens, ctx); continue; }
                        ControlFlow::Return(_) => { ctx.control_flow = Some(sig); ctx.loop_level -= 1; break; }
                    }
                }
                ctx.loop_level -= 1;
                cond_status = exec_group_tokens(&cond_tokens, ctx);
            }
            last_status
        }
        "case" => {
            if tokens.len() < 3 {
                eprintln!("sh: syntax error: case word in ... esac");
                return 1;
            }
            let word = expand_vars(&tokens[1], ctx);
            let in_pos = tokens.iter().position(|t| t == "in").unwrap_or(2);
            let body_tokens: Vec<String> = tokens[in_pos + 1..].to_vec();
            // Find matching pattern
            let mut i = 0;
            // Skip leading ; from line joining
            while i < body_tokens.len() && body_tokens[i] == ";" { i += 1; }
            let mut result = 0;
            let mut matched = false;
            while i < body_tokens.len() {
                // Skip ; from line joining
                while i < body_tokens.len() && body_tokens[i] == ";" { i += 1; }
                if i >= body_tokens.len() { break; }
                if body_tokens[i] == "esac" { break; }
                if body_tokens[i] == ";;" || body_tokens[i] == ";;&" {
                    if matched { break; }
                    i += 1;
                    continue;
                }
                // Parse pattern: pattern[|pattern...] ) commands ;;
                let close_paren = body_tokens[i..].iter().position(|t| t == ")");
                if let Some(cp) = close_paren {
                    let pattern_tokens: Vec<String> = body_tokens[i..i+cp].iter().cloned().collect();
                    // Split on | to get individual patterns
                    let mut patterns: Vec<String> = Vec::new();
                    let mut cur_pat = String::new();
                    for pt in &pattern_tokens {
                        if pt == "|" {
                            if !cur_pat.is_empty() {
                                patterns.push(cur_pat);
                                cur_pat = String::new();
                            }
                        } else {
                            if !cur_pat.is_empty() { cur_pat.push(' '); }
                            cur_pat.push_str(pt);
                        }
                    }
                    if !cur_pat.is_empty() {
                        patterns.push(cur_pat);
                    }
                    let cmd_start = i + cp + 1;
                    let double_semi = body_tokens[cmd_start..].iter().position(|t| t == ";;" || t == "esac");
                    if let Some(ds) = double_semi {
                        let cmd_tokens: Vec<String> = body_tokens[cmd_start..cmd_start+ds].to_vec();
                        if !matched {
                            for p in &patterns {
                                let expanded = expand_vars(p, ctx);
                                if glob_match(&word, &expanded) {
                                    result = exec_group_tokens(&cmd_tokens, ctx);
                                    matched = true;
                                    break;
                                }
                            }
                        }
                        i = cmd_start + ds + 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            result
        }
        "{" => {
            // Brace group: { commands; }
            let close = tokens.iter().rposition(|t| t == "}");
            if let Some(cp) = close {
                let inner = &tokens[1..cp];
                exec_group_tokens(inner, ctx)
            } else {
                eprintln!("sh: syntax error: unclosed brace group");
                1
            }
        }
        "(" => {
            // Subshell
            let close = tokens.iter().rposition(|t| t == ")");
            if let Some(cp) = close {
                let inner_tokens: Vec<String> = tokens[1..cp].to_vec();
                let cmd_str = inner_tokens.join(" ");
                let output = Command::new("sh").arg("-c").arg(&cmd_str).output();
                match output {
                    Ok(o) => o.status.code().unwrap_or(0),
                    Err(_) => {
                        exec_group_tokens(&inner_tokens, ctx)
                    }
                }
            } else {
                eprintln!("sh: syntax error: unclosed subshell");
                1
            }
        }
        "function" | "fn" => {
            if tokens.len() < 3 || tokens[2] != "{" {
                eprintln!("sh: syntax error: function name {{ body }}");
                return 1;
            }
            let name = tokens[1].clone();
            let close = tokens.iter().rposition(|t| t == "}");
            if let Some(cp) = close {
                let body_tokens = &tokens[3..cp];
                ctx.funcs.insert(name, vec![body_tokens.to_vec()]);
                0
            } else {
                eprintln!("sh: syntax error: unclosed function body");
                1
            }
        }
        _ => {
            // Regular command
            exec_group_tokens(tokens, ctx)
        }
    }
}

// ─── Helper: split tokens after a keyword ────────────────────────────────────

fn split_after_token(tokens: &[String], keyword: &str) -> (Vec<String>, Vec<String>) {
    let pos = tokens.iter().position(|t| t == keyword);
    match pos {
        Some(p) => (tokens[..p].to_vec(), tokens[p+1..].to_vec()),
        None => (tokens.to_vec(), vec![]),
    }
}

fn split_after_token_multi(tokens: &[String], keywords: &[&str]) -> (Vec<String>, Vec<String>) {
    let pos = tokens.iter().position(|t| keywords.contains(&t.as_str()));
    match pos {
        Some(p) => (tokens[..p].to_vec(), tokens[p..].to_vec()),
        None => (tokens.to_vec(), vec![]),
    }
}

fn parse_and_return_tokens(tokens: &[String]) -> Vec<String> {
    // Return tokens that form a complete command group (up to ;, &&, || at top level)
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate() {
        match t.as_str() {
            "if" | "for" | "while" | "until" | "case" | "{" | "(" | "do" => depth += 1,
            "fi" | "done" | "esac" | "}" | ")" => depth -= 1,
            ";" | "&&" | "||" | "then" | "else" | "elif" => {
                if depth == 0 {
                    return tokens[..i].to_vec();
                }
            }
            _ => {}
        }
    }
    tokens.to_vec()
}

// ─── Execute a group of tokens (simple commands or control flow) ─────────────

fn exec_group_tokens(tokens: &[String], ctx: &mut ShellContext) -> i32 {
    if tokens.is_empty() {
        return 0;
    }

    // Split by &&, ||, ;, accounting for brace and keyword depth
    // (No top-level keyword check — let splitting happen first, then each
    //  group is routed to parse_and_exec by execute_sequence if needed)
    let mut groups: Vec<(Vec<String>, &str)> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut op = "";
    let mut brace_depth: i32 = 0;
    let mut kw_depth: i32 = 0; // tracks if/fi, for/done, while/done, case/esac

    for t in tokens {
        match t.as_str() {
            "{" => { brace_depth += 1; cur.push(t.clone()); }
            "}" => { brace_depth -= 1; cur.push(t.clone()); }
            "if" | "for" | "while" | "until" | "case" => {
                kw_depth += 1;
                cur.push(t.clone());
            }
            "fi" | "done" | "esac" => {
                kw_depth -= 1;
                cur.push(t.clone());
            }
            "&&" | "||" | ";" => {
                if brace_depth > 0 || kw_depth > 0 {
                    cur.push(t.clone());
                } else {
                    if !cur.is_empty() {
                        groups.push((cur.clone(), op));
                        cur.clear();
                    }
                    op = t;
                }
            }
            _ => { cur.push(t.clone()); }
        }
    }
    if !cur.is_empty() {
        groups.push((cur, op));
    }

    if groups.is_empty() {
        return 0;
    }

    // Execute first group
    let first_op = groups[0].1;
    let first_status = if first_op == "" || first_op == ";" || (ctx.last_status == 0 && first_op == "&&") || (ctx.last_status != 0 && first_op == "||") {
        execute_sequence(&groups[0].0, ctx)
    } else {
        0
    };
    ctx.last_status = first_status;

    for i in 1..groups.len() {
        if ctx.control_flow.is_some() {
            break;
        }
        let (ref gtokens, gop) = groups[i];
        let should_run = match gop {
            "&&" => ctx.last_status == 0,
            "||" => ctx.last_status != 0,
            _ => true,
        };
        if should_run {
            ctx.last_status = execute_sequence(gtokens, ctx);
        }
    }

    ctx.last_status
}

fn execute_sequence(tokens: &[String], ctx: &mut ShellContext) -> i32 {
    if tokens.is_empty() {
        return 0;
    }

    // Check for control flow signals
    if ctx.control_flow.is_some() {
        return ctx.last_status;
    }

// Check for function definitions: name() { body }
    if tokens.len() >= 4 && tokens[1] == "(" && tokens[2] == ")" && tokens[3] == "{" {
        let close = tokens.iter().rposition(|t| t == "}");
        if let Some(cp) = close {
            let name = tokens[0].clone();
            let body_tokens = &tokens[4..cp];
            ctx.funcs.insert(name, vec![body_tokens.to_vec()]);
            // Process tokens after the closing brace
            if cp + 1 < tokens.len() {
                return exec_group_tokens(&tokens[cp + 1..], ctx);
            }
            return 0;
        }
        eprintln!("sh: syntax error: unclosed function body");
        return 1;
    }
    // Check for function/fn keyword definitions: [function|fn] name { body }
    if tokens.len() >= 4 && (tokens[0] == "function" || tokens[0] == "fn") && tokens[2] == "{" {
        return parse_and_exec(tokens, ctx);
    }

    // Check for control flow keywords (only keywords, not function names)
    let first = tokens[0].as_str();
    if matches!(first, "if" | "elif" | "for" | "while" | "until" | "case" | "{" | "(") {
        return parse_and_exec(tokens, ctx);
    }
    // Check for function definition: name() { body }
    if tokens.len() >= 4 && tokens[1] == "(" && tokens[2] == ")" && tokens[3] == "{" {
        return parse_and_exec(tokens, ctx);
    }

    // Check for pipes
    let pipe_positions: Vec<usize> = tokens.iter().enumerate()
        .filter(|(_, t)| *t == "|")
        .map(|(i, _)| i)
        .collect();

    if !pipe_positions.is_empty() {
        // Build pipeline segments
        let mut segments: Vec<Vec<String>> = Vec::new();
        let mut start = 0;
        for &pp in &pipe_positions {
            segments.push(tokens[start..pp].to_vec());
            start = pp + 1;
        }
        segments.push(tokens[start..].to_vec());

        let status = exec_pipeline(&segments, &[], false, ctx);
        ctx.last_status = status;
        return status;
    }

    // Single command
    let cmd_tokens: Vec<String> = tokens.to_vec();

    // Expand tokens
    let expanded: Vec<String> = cmd_tokens.iter()
        .flat_map(|t| {
            let e = expand_vars(t, ctx);
            glob_expand(&e)
        })
        .collect();

    if expanded.is_empty() {
        return 0;
    }

    // Parse redirects
    let mut argv = Vec::new();
    let mut redirects = Vec::new();
    let mut i = 0;
    let mut pending_fd: Option<u32> = None;
    while i < expanded.len() {
        let token = &expanded[i];
        if is_redirect_op(token) {
            let op = token.clone();
            // Compute fd: first check if there's a pending numeric fd
            let fd = pending_fd.take().unwrap_or_else(|| {
                // Try to extract fd from the op string itself (e.g. "2>>" → 2)
                let s = op.as_str();
                let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
                digits.parse::<u32>().ok().unwrap_or(
                    if op.contains('<') || op == "<<" || op == "<<<" { 0 } else { 1 }
                )
            });
            let mut target = if i + 1 < expanded.len() { expanded[i + 1].clone() } else { String::new() };
            if is_redirect_op(&target) {
                i += 1;
                continue;
            }
            let (heredoc_content, heredoc_literal) = match op.as_str() {
                "<<" => {
                    let clean = if target.starts_with('\'') || target.starts_with('"') {
                        target[1..target.len()-1].to_string()
                    } else {
                        target.clone()
                    };
                    if let Some((mut content, lit)) = ctx.pending_heredocs.remove(&clean) {
                        target = clean;
                        if !lit {
                            content = expand_vars(&content, ctx);
                        }
                        (Some(content), lit)
                    } else {
                        (None, false)
                    }
                }
                _ => (None, false),
            };
            redirects.push(Redirect {
                fd, op, target, heredoc_content,
                heredoc_literal,
            });
            i += 2;
        } else if !token.is_empty() && token.chars().all(|c| c.is_ascii_digit())
            && i + 1 < expanded.len() && is_redirect_op(&expanded[i + 1]) {
            // A numeric token preceding a redirect op — use as fd
            let fd: u32 = token.parse().unwrap_or(1);
            pending_fd = Some(fd);
            i += 1;
        } else {
            argv.push(expanded[i].clone());
            i += 1;
        }
    }

    if argv.is_empty() {
        return 0;
    }

    // Handle NAME=value assignments at the start of argv
    let mut cmd_start = 0;
    for arg in &argv {
        if let Some(eq) = arg.find('=') {
            if eq > 0 && !arg[..eq].is_empty() && arg.chars().take(eq).all(|c| c.is_alphanumeric() || c == '_') {
                let val = &arg[eq + 1..];
                let expanded = expand_vars(val, ctx);
                ctx.set_var(&arg[..eq], &expanded);
                ctx.mark_exported(&arg[..eq]);
                cmd_start += 1;
                continue;
            }
        }
        break;
    }

    // If only assignments (no command), return with status 0
    if cmd_start >= argv.len() {
        return 0;
    }

    let cmd = &argv[cmd_start];
    let cmd_args: Vec<String> = argv[cmd_start + 1..].to_vec();

    // Apply redirects for builtins/functions (fd-level)
    #[cfg(any(unix, target_arch = "riscv64"))]
    let saved_fds: Vec<(i32, i32)> = if !redirects.is_empty() {
        apply_redirects_builtin(&redirects)
    } else {
        Vec::new()
    };
    #[cfg(not(any(unix, target_arch = "riscv64")))]
    let saved_fds: Vec<(i32, i32)> = Vec::new();

    // Check for function call
    if ctx.funcs.contains_key(cmd) {
        let body = ctx.funcs.get(cmd).unwrap().clone();
        let prev_positional = ctx.positional.clone();
        ctx.positional = cmd_args;
        ctx.func_depth += 1;
        let mut last_status = 0;
        for tokens in &body {
            last_status = exec_group_tokens(tokens, ctx);
            if let Some(ControlFlow::Return(status)) = ctx.control_flow.take() {
                last_status = status;
                break;
            }
        }
        ctx.func_depth -= 1;
        ctx.positional = prev_positional;
        ctx.last_status = last_status;
        #[cfg(any(unix, target_arch = "riscv64"))]
        restore_fds(saved_fds);
        return last_status;
    }

    if let Some(status) = exec_builtin(cmd, &cmd_args, &redirects, ctx) {
        #[cfg(any(unix, target_arch = "riscv64"))]
        restore_fds(saved_fds);
        ctx.last_status = status;
        return status;
    }
    #[cfg(any(unix, target_arch = "riscv64"))]
    restore_fds(saved_fds);

    let status = execute_cmd(&argv, &redirects, false, ctx);
    ctx.last_status = status;
    status
}

// ─── Main execution entry point ──────────────────────────────────────────────

fn exec_line(line: &str, ctx: &mut ShellContext) -> i32 {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return 0;
    }
    let tokens = tokenize(trimmed);
    if tokens.is_empty() {
        return 0;
    }

    // Check for alias expansion
    if let Some(alias) = ctx.vars.get(&format!("_alias_{}", tokens[0])) {
        let mut new_tokens = tokenize(alias);
        new_tokens.extend(tokens[1..].iter().cloned());
        return exec_group_tokens(&new_tokens, ctx);
    }

    exec_group_tokens(&tokens, ctx)
}

// ─── REPL ────────────────────────────────────────────────────────────────────

fn repl(ctx: &mut ShellContext) {
    let mut input = String::new();
    loop {
        let ps1 = ctx.vars.get("PS1").cloned().unwrap_or_else(|| "posix> ".to_string());
        print!("{}", ps1);
        io::stdout().flush().ok();
        input.clear();
        if io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) {
            println!();
            run_exit_traps(ctx);
            break;
        }
        let line = input.trim().to_string();
        if line.is_empty() { continue; }

        // Handle line continuation
        if line.ends_with('\\') {
            let mut full = line[..line.len()-1].to_string();
            loop {
                print!("> ");
                io::stdout().flush().ok();
                input.clear();
                if io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) { break; }
                let next = input.trim_end().to_string();
                if next.ends_with('\\') {
                    full.push_str(&next[..next.len()-1]);
                } else {
                    full.push_str(&next);
                    break;
                }
            }
            exec_line(&full, ctx);
        } else {
            exec_line(&line, ctx);
        }
    }
}

fn run_exit_traps(ctx: &mut ShellContext) {
    if let Some(cmd) = ctx.traps.get("EXIT").cloned() {
        ctx.traps.remove("EXIT");
        exec_line(&cmd, ctx);
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut ctx = ShellContext::new();

    if args.len() > 1 {
        if args[1] == "-c" && args.len() > 2 {
            let cmd = &args[2];
            let lines = collect_complete_lines(cmd, &mut ctx);
            for line in lines {
                exec_line(&line, &mut ctx);
            }
            run_exit_traps(&mut ctx);
            exit(ctx.last_status);
        }
        // Script file
        let content = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("sh: cannot open '{}': {}", args[1], e);
            exit(1);
        });
        if args.len() > 2 {
            ctx.positional = args[2..].to_vec();
        }
        let lines = collect_complete_lines(&content, &mut ctx);
        for line in lines {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            exec_line(&trimmed, &mut ctx);
        }
        run_exit_traps(&mut ctx);
        exit(ctx.last_status);
    }

    // Non-interactive stdin
    if !io::stdin().is_terminal() {
        let content: Vec<String> = io::stdin().lines().filter_map(|l| l.ok()).collect();
        let content = content.join("\n");
        let lines = collect_complete_lines(&content, &mut ctx);
        for line in lines {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            exec_line(&trimmed, &mut ctx);
        }
        run_exit_traps(&mut ctx);
        exit(ctx.last_status);
    }

    // Interactive REPL
    repl(&mut ctx);
}