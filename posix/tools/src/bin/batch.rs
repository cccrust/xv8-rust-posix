use std::fs;
use std::io::{self, BufRead};

fn main() {
    let spool = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/.batch";
    let _ = fs::create_dir_all(&spool);

    let mut script = String::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => { script.push_str(&l); script.push('\n'); }
            Err(_) => break,
        }
    }
    let job_id = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    fs::write(format!("{}/{}", spool, job_id), &script).unwrap();
    eprintln!("batch job {}", job_id);
}
