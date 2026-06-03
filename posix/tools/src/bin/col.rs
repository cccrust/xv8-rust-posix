use std::io::{self, BufRead, Write};

fn main() {
    for line in io::stdin().lock().lines() {
        match line {
            Ok(l) => {
                let processed = process_line(&l);
                println!("{}", processed);
            }
            Err(_) => break,
        }
    }
}

fn process_line(s: &str) -> String {
    let mut out = String::new();
    let mut in_rev = false;
    let mut rev_buf = String::new();
    for c in s.chars() {
        match c {
            '\u{008D}' => { in_rev = true; rev_buf.clear(); } // SO (shift out) - start reverse
            '\u{008E}' => { // SI (shift in) - end reverse
                in_rev = false;
                out.push_str(&rev_buf.chars().rev().collect::<String>());
                rev_buf.clear();
            }
            '\u{0008}' => { // backspace
                out.pop();
            }
            '\u{0007}' | '\r' => {} // bell, CR - skip
            c if in_rev => rev_buf.push(c),
            c => out.push(c),
        }
    }
    if in_rev {
        out.push_str(&rev_buf.chars().rev().collect::<String>());
    }
    out
}