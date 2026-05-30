use std::io::{self, BufRead};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tsort <file>");
        std::process::exit(1);
    }
    let file = &args[1];
    let lines = read_lines(file);
    let pairs: Vec<(String, String)> = lines.chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect();

    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_deg: HashMap<String, usize> = HashMap::new();
    for (a, b) in &pairs {
        adj.entry(a.clone()).or_default().push(b.clone());
        in_deg.entry(a.clone()).or_insert(0);
        *in_deg.entry(b.clone()).or_insert(0) += 1;
    }

    let mut queue: Vec<String> = in_deg.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(k, _)| k.clone())
        .collect();
    queue.sort_by(|a, b| b.cmp(a));

    let mut result = Vec::new();
    while let Some(node) = queue.pop() {
        result.push(node.clone());
        if let Some(neighbors) = adj.remove(&node) {
            for n in neighbors {
                if let Some(deg) = in_deg.get_mut(&n) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(n.clone());
                        queue.sort_by(|a, b| b.cmp(a));
                    }
                }
            }
        }
    }

    for node in &result {
        println!("{}", node);
    }
}

fn read_lines(path: &str) -> Vec<String> {
    if path == "-" {
        io::stdin().lock().lines().filter_map(|l| l.ok()).collect()
    } else {
        std::fs::read_to_string(path).unwrap_or_default().lines().map(|l| l.to_string()).collect()
    }
}
