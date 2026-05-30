use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut dest = String::new();
    let mut copies = 1;
    let mut i = 1;
    let mut files: Vec<&str> = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "-d" => { i += 1; if i < args.len() { dest = args[i].clone(); } }
            "-n" => { i += 1; if i < args.len() { copies = args[i].parse().unwrap_or(1); } }
            _ => { files.push(&args[i]); }
        }
        i += 1;
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    if files.is_empty() {
        files.push("-");
    }

    for fname in files {
        let content: Vec<u8> = if fname == "-" {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf).unwrap_or_default();
            buf
        } else {
            std::fs::read(fname).unwrap_or_default()
        };

        if dest.is_empty() || dest == "-" {
            // Print to stdout
            for _ in 0..copies {
                io::stdout().write_all(&content).unwrap();
            }
        } else {
            // Write to spool directory
            let spool = format!("{}/.lp/{}", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()), dest);
            let _ = std::fs::create_dir_all(&spool);
            let job_id = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
            let spool_file = format!("{}/job_{}", spool, job_id);
            let header = format!("From: {}\nFile: {}\nCopies: {}\n\n", user, fname, copies);
            let mut out = header.into_bytes();
            out.extend_from_slice(&content);
            std::fs::write(&spool_file, &out).unwrap_or_default();
            eprintln!("lp: request id is {}-{} (1 file(s))", dest, job_id);
        }
    }
}
