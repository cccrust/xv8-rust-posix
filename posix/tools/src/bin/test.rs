use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // POSIX test evaluates expressions and returns 0 (true) or 1 (false)

    // Handle [ variant: if last arg is "]", ignore it
    let arg_start = 1;
    let mut arg_end = args.len();
    if args.len() > 1 && args[args.len() - 1] == "]" {
        arg_end = args.len() - 1;
        // First arg should be "["
        if args[1] == "]" {
            eprintln!("test: missing argument after ']'");
            exit(2);
        }
    }

    let test_args: Vec<&str> = args[arg_start..arg_end].iter().map(String::as_str).collect();

    if test_args.is_empty() {
        exit(1); // no args → false
    }

    // -n STRING (string non-empty)
    if test_args.len() == 2 && test_args[0] == "-n" {
        exit(if test_args[1].is_empty() { 1 } else { 0 });
    }

    // -z STRING (string empty)
    if test_args.len() == 2 && test_args[0] == "-z" {
        exit(if test_args[1].is_empty() { 0 } else { 1 });
    }

    // STRING = STRING
    if test_args.len() == 3 && test_args[1] == "=" {
        exit(if test_args[0] == test_args[2] { 0 } else { 1 });
    }

    // STRING != STRING
    if test_args.len() == 3 && test_args[1] == "!=" {
        exit(if test_args[0] != test_args[2] { 0 } else { 1 });
    }

    // INTEGER -eq INTEGER
    if test_args.len() == 3 && test_args[1] == "-eq" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a == b { 0 } else { 1 });
    }

    // INTEGER -ne INTEGER
    if test_args.len() == 3 && test_args[1] == "-ne" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a != b { 0 } else { 1 });
    }

    // INTEGER -lt INTEGER
    if test_args.len() == 3 && test_args[1] == "-lt" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a < b { 0 } else { 1 });
    }

    // INTEGER -le INTEGER
    if test_args.len() == 3 && test_args[1] == "-le" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a <= b { 0 } else { 1 });
    }

    // INTEGER -gt INTEGER
    if test_args.len() == 3 && test_args[1] == "-gt" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a > b { 0 } else { 1 });
    }

    // INTEGER -ge INTEGER
    if test_args.len() == 3 && test_args[1] == "-ge" {
        let a: i64 = test_args[0].parse().unwrap_or(0);
        let b: i64 = test_args[2].parse().unwrap_or(0);
        exit(if a >= b { 0 } else { 1 });
    }

    // -e PATH (file exists)
    if test_args.len() == 2 && test_args[0] == "-e" {
        exit(if Path::new(&test_args[1]).exists() { 0 } else { 1 });
    }

    // -f PATH (regular file)
    if test_args.len() == 2 && test_args[0] == "-f" {
        exit(if Path::new(&test_args[1]).is_file() { 0 } else { 1 });
    }

    // -d PATH (directory)
    if test_args.len() == 2 && test_args[0] == "-d" {
        exit(if Path::new(&test_args[1]).is_dir() { 0 } else { 1 });
    }

    // ! EXPR
    if test_args.len() >= 2 && test_args[0] == "!" {
        // For now, only support ! -f, ! -d, ! -e
        // Use the same args without !
        let inner: Vec<&str> = test_args[1..].to_vec();
        // Recurse conceptually
        // Simplified: just invert the result of the inner test
        let _inner_args: Vec<String> = std::iter::once(String::new())
            .chain(inner.iter().map(|s| s.to_string()))
            .collect();
        // We can't easily re-parse; handle simple cases
        if inner.len() == 2 && inner[0] == "-e" {
            exit(if Path::new(&inner[1]).exists() { 1 } else { 0 });
        }
        if inner.len() == 2 && inner[0] == "-f" {
            exit(if Path::new(&inner[1]).is_file() { 1 } else { 0 });
        }
        if inner.len() == 2 && inner[0] == "-d" {
            exit(if Path::new(&inner[1]).is_dir() { 1 } else { 0 });
        }
        exit(1);
    }

    // Single string (non-empty check)
    if test_args.len() == 1 {
        exit(if test_args[0].is_empty() { 1 } else { 0 });
    }

    // Unknown expression
    eprintln!("test: unknown expression");
    exit(2);
}
