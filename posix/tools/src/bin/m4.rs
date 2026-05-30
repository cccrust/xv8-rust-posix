use std::collections::HashMap;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut defs: HashMap<String, String> = HashMap::new();
    let mut diverts: Vec<String> = Vec::new();
    let mut cur_divert: usize = 0;
    let mut output = String::new();

    let files: Vec<&str> = if args.len() > 1 {
        args[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec!["-"]
    };

    for f in files {
        let content = if f == "-" {
            let mut buf = String::new();
            std::io::stdin().lock().read_to_string(&mut buf).ok();
            buf
        } else {
            std::fs::read_to_string(f).unwrap_or_default()
        };

        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();
        while i < chars.len() {
            // Skip comment lines (dnl)
            if chars[i] == 'd' && chars.len() > i + 3 && content[i..].starts_with("dnl") {
                while i < chars.len() && chars[i] != '\n' { i += 1; }
                i += 1;
                continue;
            }

            // define(name, value) - handle both with and without backtick quotes
            if content[i..].starts_with("define(") || content[i..].starts_with("define(`") {
                if content[i..].starts_with("define(`") { i += 8; } else { i += 7; }
                let name = read_until(&chars, &mut i, |c| c == '\'' || c == ',' || c == ')');
                let val = read_until(&chars, &mut i, |c| c == '\'' || c == ')' || c == ',');
                defs.insert(name.trim().to_string(), val.trim().to_string());
                continue;
            }

            // undefine(name)
            if content[i..].starts_with("undefine(") || content[i..].starts_with("undefine(`") {
                if content[i..].starts_with("undefine(`") { i += 10; } else { i += 9; }
                let name = read_until(&chars, &mut i, |c| c == '\'' || c == ')');
                defs.remove(name.trim());
                continue;
            }

            // ifdef(name, true-branch, false-branch)
            if content[i..].starts_with("ifdef(") || content[i..].starts_with("ifdef(`") {
                if content[i..].starts_with("ifdef(`") { i += 7; } else { i += 6; }
                let name = read_until(&chars, &mut i, |c| c == '\'' || c == ',');
                let true_br = read_until(&chars, &mut i, |c| c == ',' || c == ')');
                let false_br = if i < chars.len() && chars[i] == ',' {
                    i += 1;
                    read_until(&chars, &mut i, |c| c == ')')
                } else { String::new() };
                if defs.contains_key(name.trim()) {
                    output.push_str(true_br.trim());
                } else if !false_br.trim().is_empty() {
                    output.push_str(false_br.trim());
                }
                continue;
            }

            // include(file)
            if content[i..].starts_with("include(") {
                i += 8;
                let fname = read_until(&chars, &mut i, |c| c == ')');
                if let Ok(inc) = std::fs::read_to_string(fname.trim()) {
                    output.push_str(&inc);
                }
                continue;
            }

            // divert(n)
            if content[i..].starts_with("divert(") {
                i += 7;
                let n_str = read_until(&chars, &mut i, |c| c == ')');
                let n: usize = n_str.trim().parse().unwrap_or(0);
                if n >= diverts.len() { diverts.resize(n + 1, String::new()); }
                cur_divert = n;
                continue;
            }

            // undivert(n)
            if content[i..].starts_with("undivert(") {
                i += 9;
                let n_str = read_until(&chars, &mut i, |c| c == ')');
                let n: usize = n_str.trim().parse().unwrap_or(0);
                if n < diverts.len() { output.push_str(&diverts[n]); }
                continue;
            }

            // eval(expr)
            if content[i..].starts_with("eval(") {
                i += 5;
                let expr = read_until(&chars, &mut i, |c| c == ')');
                output.push_str(&eval_expr(&expr));
                continue;
            }

            // len(str)
            if content[i..].starts_with("len(") {
                i += 4;
                let s = read_until(&chars, &mut i, |c| c == ')');
                output.push_str(&s.trim().len().to_string());
                continue;
            }

            // substr(str, start, len)
            if content[i..].starts_with("substr(") {
                i += 7;
                let s = read_until(&chars, &mut i, |c| c == ',');
                let start_str = read_until(&chars, &mut i, |c| c == ',');
                let len_str = read_until(&chars, &mut i, |c| c == ')');
                let start: usize = start_str.trim().parse().unwrap_or(0);
                let len: usize = len_str.trim().parse().unwrap_or(0);
                let result: String = s.trim().chars().skip(start).take(len).collect();
                output.push_str(&result);
                continue;
            }

            // Expand macros in free text
            if chars[i] == '\n' {
                if cur_divert == 0 { output.push('\n'); }
                else if cur_divert < diverts.len() { diverts[cur_divert].push('\n'); }
                i += 1;
                continue;
            }

            if chars[i].is_alphanumeric() || chars[i] == '_' {
                let mut word = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    word.push(chars[i]);
                    i += 1;
                }
                if defs.contains_key(&word) {
                    let val = defs.get(&word).unwrap().clone();
                    if cur_divert == 0 { output.push_str(&val); }
                    else if cur_divert < diverts.len() { diverts[cur_divert].push_str(&val); }
                } else {
                    if cur_divert == 0 { output.push_str(&word); }
                    else if cur_divert < diverts.len() { diverts[cur_divert].push_str(&word); }
                }
            } else {
                let c = chars[i];
                if cur_divert == 0 { output.push(c); }
                else if cur_divert < diverts.len() { diverts[cur_divert].push(c); }
                i += 1;
            }
        }
    }
    print!("{}", output);
}

fn read_until(chars: &[char], i: &mut usize, stop: impl Fn(char) -> bool) -> String {
    let mut s = String::new();
    while *i < chars.len() && !stop(chars[*i]) {
        s.push(chars[*i]);
        *i += 1;
    }
    if *i < chars.len() { *i += 1; }
    while *i < chars.len() && (chars[*i] == ',' || chars[*i] == ')' || chars[*i].is_whitespace()) {
        *i += 1;
    }
    s
}

fn eval_expr(expr: &str) -> String {
    let s = expr.trim();
    let mut result = 0i64;
    let mut op = '+';
    let mut num = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() || (chars[i] == '-' && num.is_empty()) {
            num.push(chars[i]);
        } else if chars[i] == '+' || chars[i] == '-' || chars[i] == '*' || chars[i] == '/' || chars[i] == '%' {
            if !num.is_empty() {
                let n: i64 = num.parse().unwrap_or(0);
                match op {
                    '+' => result += n, '-' => result -= n, '*' => result *= n,
                    '/' => if n != 0 { result /= n; },
                    '%' => if n != 0 { result %= n; },
                    _ => result = n,
                }
                num.clear();
            }
            op = chars[i];
        }
        i += 1;
    }
    if !num.is_empty() {
        let n: i64 = num.parse().unwrap_or(0);
        match op {
            '+' => result += n, '-' => result -= n, '*' => result *= n,
            '/' => if n != 0 { result /= n; },
            '%' => if n != 0 { result %= n; },
            _ => result = n,
        }
    }
    result.to_string()
}
