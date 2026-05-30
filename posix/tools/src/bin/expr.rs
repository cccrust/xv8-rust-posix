fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: expr <operand> <operator> <operand> [...]");
        std::process::exit(1);
    }
    let tokens: Vec<String> = args[1..].to_vec();
    let result = eval(&tokens);
    println!("{}", result);
    if result == 0 {
        std::process::exit(1);
    }
}

fn eval(tokens: &[String]) -> i64 {
    let mut idx = 0;
    parse_expr(tokens, &mut idx)
}

fn parse_expr(tokens: &[String], idx: &mut usize) -> i64 {
    let mut left = parse_term(tokens, idx);
    while *idx < tokens.len() {
        match tokens[*idx].as_str() {
            "|" => {
                *idx += 1;
                let right = parse_term(tokens, idx);
                if left == 0 {
                    left = right;
                }
            }
            "&" => {
                *idx += 1;
                let right = parse_term(tokens, idx);
                if left != 0 && right != 0 {
                    left = right;
                } else {
                    left = 0;
                }
            }
            _ => break,
        }
    }
    left
}

fn parse_term(tokens: &[String], idx: &mut usize) -> i64 {
    let mut left = parse_cmp(tokens, idx);
    while *idx < tokens.len() {
        match tokens[*idx].as_str() {
            "+" => {
                *idx += 1;
                let right = parse_cmp(tokens, idx);
                left = left.wrapping_add(right);
            }
            "-" => {
                *idx += 1;
                let right = parse_cmp(tokens, idx);
                left = left.wrapping_sub(right);
            }
            _ => break,
        }
    }
    left
}

fn parse_cmp(tokens: &[String], idx: &mut usize) -> i64 {
    let left = parse_factor(tokens, idx);
    if *idx < tokens.len() {
        match tokens[*idx].as_str() {
            "=" | "==" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left == right { 1 } else { 0 };
            }
            "!=" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left != right { 1 } else { 0 };
            }
            ">" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left > right { 1 } else { 0 };
            }
            ">=" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left >= right { 1 } else { 0 };
            }
            "<" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left < right { 1 } else { 0 };
            }
            "<=" => {
                *idx += 1;
                let right = parse_factor(tokens, idx);
                return if left <= right { 1 } else { 0 };
            }
            _ => {}
        }
    }
    left
}

fn parse_factor(tokens: &[String], idx: &mut usize) -> i64 {
    let left = parse_primary(tokens, idx);
    if *idx < tokens.len() {
        match tokens[*idx].as_str() {
            "*" => {
                *idx += 1;
                let right = parse_primary(tokens, idx);
                return left.wrapping_mul(right);
            }
            "/" => {
                *idx += 1;
                let right = parse_primary(tokens, idx);
                if right == 0 { return 0; }
                return left / right;
            }
            "%" => {
                *idx += 1;
                let right = parse_primary(tokens, idx);
                if right == 0 { return 0; }
                return left % right;
            }
            _ => {}
        }
    }
    left
}

fn parse_primary(tokens: &[String], idx: &mut usize) -> i64 {
    if *idx >= tokens.len() { return 0; }
    let tok = &tokens[*idx];
    *idx += 1;
    if let Ok(n) = tok.parse::<i64>() {
        n
    } else {
        match tok.as_str() {
            "(" => {
                let val = parse_expr(tokens, idx);
                if *idx < tokens.len() && tokens[*idx] == ")" {
                    *idx += 1;
                }
                val
            }
            _ => 0,
        }
    }
}
