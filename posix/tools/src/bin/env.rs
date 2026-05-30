fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    let mut ignore_env = false;
    let mut unset_vars: Vec<String> = Vec::new();
    let mut set_vars: Vec<(String, String)> = Vec::new();

    while i < args.len() && args[i].starts_with('-') {
        match args[i].as_str() {
            "-i" | "-" => {
                ignore_env = true;
                i += 1;
            }
            "-u" => {
                i += 1;
                if i < args.len() {
                    unset_vars.push(args[i].clone());
                    i += 1;
                }
            }
            "--" => {
                i += 1;
                break;
            }
            _ => break,
        }
    }

    while i < args.len() {
        if let Some(eq_pos) = args[i].find('=') {
            let key = args[i][..eq_pos].to_string();
            let val = args[i][eq_pos + 1..].to_string();
            set_vars.push((key, val));
            i += 1;
        } else {
            break;
        }
    }

    let cmd: Vec<String> = args[i..].to_vec();

    if cmd.is_empty() {
        for (key, val) in std::env::vars() {
            println!("{}={}", key, val);
        }
        return;
    }

    let mut command = std::process::Command::new(&cmd[0]);
    command.args(&cmd[1..]);

    if ignore_env {
        command.env_clear();
    }

    for (k, v) in &set_vars {
        command.env(k, v);
    }

    for k in &unset_vars {
        command.env_remove(k);
    }

    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(0)),
        Err(e) => {
            eprintln!("env: {}", e);
            std::process::exit(127);
        }
    }
}
