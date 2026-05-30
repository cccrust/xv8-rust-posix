use std::fs;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spool = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()) + "/.at";
    let _ = fs::create_dir_all(&spool);

    if args.len() < 2 {
        eprintln!("Usage: at [-l|-r jobid] time");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "-l" => {
            list_jobs(&spool);
        }
        "-r" => {
            if args.len() < 3 {
                eprintln!("at: missing job id");
                std::process::exit(1);
            }
            let jobfile = format!("{}/{}", spool, args[2]);
            let _ = fs::remove_file(&jobfile);
        }
        time_spec => {
            // Read script from stdin
            let mut script = String::new();
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => { script.push_str(&l); script.push('\n'); }
                    Err(_) => break,
                }
            }
            let (typ, spec) = if time_spec == "now" { ("now", "") } else { ("at", time_spec) };
            let job_id = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
            let job = format!("{}|{}|{}\n", typ, spec, script);
            fs::write(format!("{}/{}", spool, job_id), &job).unwrap();
            eprintln!("job {} at {}", job_id, time_spec);
        }
    }
}

fn list_jobs(spool: &str) {
    let dir = match fs::read_dir(spool) {
        Ok(d) => d,
        Err(_) => return,
    };
    for entry in dir {
        if let Ok(e) = entry {
            if let Ok(content) = fs::read_to_string(e.path()) {
                if let Some((typ, rest)) = content.split_once('|') {
                    if let Some((spec, _script)) = rest.split_once('|') {
                        println!("{}\t{}\t{}", e.file_name().to_string_lossy(), typ, spec);
                    }
                }
            }
        }
    }
}
