use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        eprintln!("diff: invalid option -- '{}'", args[i]);
        std::process::exit(1);
    }

    if i + 2 > args.len() {
        eprintln!("usage: diff file1 file2");
        std::process::exit(1);
    }

    let path1 = Path::new(&args[i]);
    let path2 = Path::new(&args[i + 1]);

    let text1 = fs::read_to_string(path1).unwrap_or_else(|e| {
        eprintln!("diff: {}: {}", path1.display(), e);
        std::process::exit(1);
    });
    let text2 = fs::read_to_string(path2).unwrap_or_else(|e| {
        eprintln!("diff: {}: {}", path2.display(), e);
        std::process::exit(1);
    });

    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();

    let ops = lcs_diff(&lines1, &lines2);

    if ops.is_empty() { return; }

    // Check if all ops are Same (files are identical)
    let has_changes = ops.iter().any(|op| !matches!(op, DiffOp::Same(_)));
    if !has_changes { return; }

    println!("--- {}", path1.display());
    println!("+++ {}", path2.display());

    // Emit hunks: find ranges with changes + context
    let mut pos: usize = 0;
    while pos < ops.len() {
        // Skip Same ops
        while pos < ops.len() && matches!(&ops[pos], DiffOp::Same(_)) {
            pos += 1;
        }
        if pos >= ops.len() { break; }

        // Find end of this change region (include up to 3 context Same after)
        let mut end = pos + 1;
        let mut ctx = 0;
        while end < ops.len() && ctx < 3 {
            match &ops[end] {
                DiffOp::Same(_) => ctx += 1,
                _ => ctx = 0,
            }
            end += 1;
        }
        if ctx >= 3 { end -= 3; }

        // Include up to 3 context Same before
        let mut hunk_start = pos;
        ctx = 0;
        while hunk_start > 0 && ctx < 3 {
            hunk_start -= 1;
            match &ops[hunk_start] {
                DiffOp::Same(_) => ctx += 1,
                _ => { ctx = 0; }
            }
        }

        // Compute line numbers for this hunk
        let a_start = compute_a_line(&ops, hunk_start, &lines1);
        let b_start = compute_b_line(&ops, hunk_start, &lines2);

        let mut a_count = 0usize;
        let mut b_count = 0usize;
        for op in &ops[hunk_start..end] {
            match op {
                DiffOp::Same(_) => { a_count += 1; b_count += 1; }
                DiffOp::Delete(_) => { a_count += 1; }
                DiffOp::Insert(_) => { b_count += 1; }
            }
        }
        if a_count == 0 { a_count = 1; }
        if b_count == 0 { b_count = 1; }

        println!("@@ -{},{} +{},{} @@", a_start, a_count, b_start, b_count);
        for op in &ops[hunk_start..end] {
            match op {
                DiffOp::Same(l) => println!(" {}", l),
                DiffOp::Delete(l) => println!("-{}", l),
                DiffOp::Insert(l) => println!("+{}", l),
            }
        }

        pos = end;
    }

    std::process::exit(1);
}

fn compute_a_line(ops: &[DiffOp], up_to: usize, lines: &[&str]) -> usize {
    let mut line = 1usize;
    for (idx, op) in ops.iter().enumerate() {
        if idx >= up_to { break; }
        match op {
            DiffOp::Same(_) | DiffOp::Delete(_) => line += 1,
            DiffOp::Insert(_) => {}
        }
    }
    line.min(lines.len().max(1))
}

fn compute_b_line(ops: &[DiffOp], up_to: usize, lines: &[&str]) -> usize {
    let mut line = 1usize;
    for (idx, op) in ops.iter().enumerate() {
        if idx >= up_to { break; }
        match op {
            DiffOp::Same(_) | DiffOp::Insert(_) => line += 1,
            DiffOp::Delete(_) => {}
        }
    }
    line.min(lines.len().max(1))
}

#[derive(Debug)]
enum DiffOp<'a> {
    Same(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

fn lcs_diff<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<DiffOp<'a>> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut ops: Vec<DiffOp<'a>> = Vec::with_capacity(m + n);
    let mut i = m;
    let mut j = n;
    let mut temp = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
            temp.push(DiffOp::Same(a[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            temp.push(DiffOp::Insert(b[j - 1]));
            j -= 1;
        } else {
            temp.push(DiffOp::Delete(a[i - 1]));
            i -= 1;
        }
    }

    while let Some(op) = temp.pop() {
        ops.push(op);
    }

    ops
}
