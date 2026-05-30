use std::collections::HashMap;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut vars: HashMap<String, String> = HashMap::new();
    let scale: usize = 0;
    let ibase: u32 = 10;
    let obase: u32 = 10;

    if args.len() > 1 {
        let expr = args[1..].join(" ");
        if let Some(result) = eval_line(&expr, &mut vars, scale, ibase, obase) {
            println!("{}", result);
        }
    } else {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap_or_default();
            let line = line.trim().to_string();
            if line.is_empty() { continue; }
            if line == "quit" { break; }
            let result = eval_line(&line, &mut vars, scale, ibase, obase);
            if let Some(r) = result {
                println!("{}", r);
            }
        }
    }
}

fn eval_line(line: &str, vars: &mut HashMap<String, String>, _scale: usize, _ibase: u32, obase: u32) -> Option<String> {
    let line = line.trim();
    if line == "quit" { return None; }
    if line.starts_with("scale=") {
        let n: usize = line[6..].trim().parse().unwrap_or(0);
        return Some(n.to_string());
    }
    if let Some(eq) = line.find('=') {
        let name = line[..eq].trim().to_string();
        let expr = line[eq+1..].trim();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic()) {
            if let Some(val) = eval_line(expr, vars, _scale, _ibase, obase) {
                vars.insert(name, val.clone());
                return Some(val);
            }
        }
    }
    let result = eval_arith(line, vars);
    if let Some(ref n) = result {
        if obase != 10 {
            let n_int: i64 = n.parse().unwrap_or(0);
            return match obase {
                16 => Some(format!("{:X}", n_int)),
                8 => Some(format!("{:o}", n_int)),
                2 => Some(format!("{:b}", n_int)),
                _ => result,
            };
        }
    }
    result
}

fn eval_arith(expr: &str, vars: &HashMap<String, String>) -> Option<String> {
    let mut s = expr.to_string();
    for (name, val) in vars.iter() {
        s = s.replace(name.as_str(), val.as_str());
    }
    let tokens = tokenize(&s);
    let rpn = shunt(&tokens);
    let result = eval_rpn(&rpn);
    result.map(|n| n.to_string())
}

fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
        } else {
            if !num.is_empty() {
                tokens.push(num.clone());
                num.clear();
            }
            if !c.is_whitespace() {
                tokens.push(c.to_string());
            }
        }
    }
    if !num.is_empty() {
        tokens.push(num);
    }
    tokens
}

fn shunt(tokens: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    let mut ops: Vec<&str> = Vec::new();
    let prec = |op: &str| -> u32 { match op { "+" | "-" => 1, "*" | "/" | "%" => 2, "^" => 3, _ => 0 } };
    for t in tokens {
        if t.chars().all(|c| c.is_ascii_digit() || c == '.') {
            output.push(t.clone());
        } else if t == "(" {
            ops.push("(");
        } else if t == ")" {
            while let Some(op) = ops.last() {
                if *op == "(" { ops.pop(); break; }
                output.push(ops.pop().unwrap().to_string());
            }
        } else {
            while let Some(op) = ops.last() {
                if *op == "(" { break; }
                if prec(op) >= prec(t) {
                    output.push(ops.pop().unwrap().to_string());
                } else { break; }
            }
            ops.push(t);
        }
    }
    while let Some(op) = ops.pop() {
        if op != "(" { output.push(op.to_string()); }
    }
    output
}

fn eval_rpn(tokens: &[String]) -> Option<i64> {
    let mut stack: Vec<i64> = Vec::new();
    for t in tokens {
        if let Ok(n) = t.parse::<i64>() {
            stack.push(n);
        } else {
            let b = stack.pop().unwrap_or(0);
            let a = stack.pop().unwrap_or(0);
            match t.as_str() {
                "+" => stack.push(a + b),
                "-" => stack.push(a - b),
                "*" => stack.push(a * b),
                "/" => stack.push(if b != 0 { a / b } else { 0 }),
                "%" => stack.push(if b != 0 { a % b } else { 0 }),
                "^" => stack.push(a.pow(b as u32)),
                _ => {}
            }
        }
    }
    stack.pop()
}
