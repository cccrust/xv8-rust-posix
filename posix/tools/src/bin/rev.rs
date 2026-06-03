use std::io::{self, BufRead, Write};

fn main() {
    for line in io::stdin().lock().lines() {
        match line {
            Ok(l) => {
                let reversed: String = l.chars().rev().collect();
                println!("{}", reversed);
            }
            Err(_) => break,
        }
    }
}