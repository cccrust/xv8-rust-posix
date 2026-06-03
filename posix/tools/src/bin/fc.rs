use std::fs;
use std::process::Command;

const DEFAULT_HISTFILE: &str = ".sh_history";

fn histfile_path() -> String {
    std::env::var("HISTFILE")
        .or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            if home.is_empty() {
                Err(std::env::VarError::NotPresent)
            } else {
                Ok(format!("{}/{}", home, DEFAULT_HISTFILE))
            }
        })
        .unwrap_or_else(|_| DEFAULT_HISTFILE.to_string())
}

fn read_history(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .map(|c| c.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    let mut flag_l = false;
    let mut flag_r = false;
    let mut flag_n = false;
    let mut flag_s = false;
    let mut editor: Option<String> = None;

    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-l" => { flag_l = true; i += 1; }
            "-r" => { flag_r = true; i += 1; }
            "-n" => { flag_n = true; i += 1; }
            "-s" => { flag_s = true; i += 1; }
            "-e" => {
                i += 1;
                if i < args.len() {
                    editor = Some(args[i].clone());
                    i += 1;
                } else {
                    eprintln!("fc: -e requires an editor argument");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("fc: unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let histfile = histfile_path();
    let history = read_history(&histfile);
    let total = history.len();

    let first = if i < args.len() {
        parse_num(&args[i], total)
    } else {
        if total > 0 { total } else { 0 }
    };
    let last = if i + 1 < args.len() {
        parse_num(&args[i + 1], total)
    } else {
        first
    };

    if flag_s {
        let cmd_idx = if first > 0 && first <= total { first } else { total };
        let cmd = if cmd_idx > 0 && cmd_idx <= total {
            history[cmd_idx - 1].clone()
        } else {
            String::new()
        };
        if !cmd.is_empty() {
            if let Some(fname) = std::env::args().find(|a| a.contains('=')) {
                let parts: Vec<&str> = fname.split('=').collect();
                if parts.len() == 2 {
                    let old = parts[0];
                    let new = parts[1];
                    let replaced = cmd.replace(old, new);
                    println!("{}", replaced);
                    let _ = Command::new("sh").arg("-c").arg(&replaced).status();
                    return;
                }
            }
            println!("{}", cmd);
            let _ = Command::new("sh").arg("-c").arg(&cmd).status();
        }
        return;
    }

    if editor.is_some() {
        let tmpfile = "/tmp/fc_edit.txt".to_string();
        if let Ok(mut f) = fs::File::create(&tmpfile) {
            use std::io::Write;
            for j in std::cmp::max(1, first)..=std::cmp::min(last, total) {
                if j <= total {
                    writeln!(f, "{}", history[j - 1]).ok();
                }
            }
            drop(f);
            let ed = editor.unwrap_or_else(|| "ed".to_string());
            let status = Command::new(&ed).arg(&tmpfile).status();
            if let Ok(Some(0)) = status.map(|s| s.code()) {
                if let Ok(edited) = fs::read_to_string(&tmpfile) {
                    for line in edited.lines() {
                        if !line.trim().is_empty() {
                            println!("{}", line);
                            let _ = Command::new("sh").arg("-c").arg(line).status();
                        }
                    }
                }
            }
        }
        return;
    }

    // Default: list history
    let start = std::cmp::max(1, first);
    let end = std::cmp::min(last, total);
    if start == 0 || end == 0 {
        return;
    }
    let mut indices: Vec<usize> = (start..=end).collect();
    if flag_r {
        indices.reverse();
    }
    for j in indices {
        if j <= total {
            if flag_n {
                println!("{}", history[j - 1]);
            } else {
                println!("{}\t{}", j, history[j - 1]);
            }
        }
    }
}

fn parse_num(s: &str, total: usize) -> usize {
    if s.starts_with('-') {
        let n: usize = s[1..].parse().unwrap_or(0);
        if n == 0 { return 0; }
        if total >= n { total - n + 1 } else { 1 }
    } else {
        s.parse().unwrap_or(total)
    }
}
