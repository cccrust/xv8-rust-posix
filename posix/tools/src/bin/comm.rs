use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        for c in args[i][1..].chars() {
            match c {
                '1' | '2' | '3' => {} // column suppression
                _ => { eprintln!("comm: invalid option -- '{}'", c); std::process::exit(1); }
            }
        }
        i += 1;
    }

    let file1 = if i < args.len() { &args[i] } else { "-" }; i += 1;
    let file2 = if i < args.len() { &args[i] } else { "-" };

    let lines1 = read_lines(file1);
    let lines2 = read_lines(file2);

    let mut i = 0usize;
    let mut j = 0usize;

    while i < lines1.len() || j < lines2.len() {
        if i >= lines1.len() {
            println!("\t\t{}", lines2[j]);
            j += 1;
        } else if j >= lines2.len() {
            println!("{}", lines1[i]);
            i += 1;
        } else {
            match lines1[i].cmp(&lines2[j]) {
                std::cmp::Ordering::Less => {
                    println!("{}", lines1[i]);
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    println!("\t\t{}", lines2[j]);
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    println!("\t{}", lines1[i]);
                    i += 1;
                    j += 1;
                }
            }
        }
    }
}

fn read_lines(name: &str) -> Vec<String> {
    if name == "-" {
        io::stdin().lock().lines().map(|l| l.unwrap_or_default()).collect()
    } else {
        let content = std::fs::read_to_string(Path::new(name)).unwrap_or_default();
        content.lines().map(|l| l.to_string()).collect()
    }
}
