use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let cron_dir = format!("{}/.cron", home);
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let crontab_file = format!("{}/{}", cron_dir, user);

    if args.len() < 2 {
        eprintln!("Usage: crontab [-l|-e|-r] [file]");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "-l" => {
            if let Ok(content) = fs::read_to_string(&crontab_file) {
                print!("{}", content);
            }
        }
        "-e" => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&crontab_file)
                .status();
            if status.is_err() || !status.unwrap().success() {
                eprintln!("crontab: editor {} failed", editor);
                std::process::exit(1);
            }
        }
        "-r" => {
            let _ = fs::remove_file(&crontab_file);
        }
        file_arg => {
            let _ = fs::create_dir_all(&cron_dir);
            match fs::copy(file_arg, &crontab_file) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("crontab: {}: {}", file_arg, e);
                    std::process::exit(1);
                }
            }
        }
    }
}
