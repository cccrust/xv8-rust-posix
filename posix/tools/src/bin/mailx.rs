use std::fs;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let mbox = format!("{}/mbox", home);
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    if args.len() < 2 {
        // Read mode: list messages
        let content = fs::read_to_string(&mbox).unwrap_or_default();
        let msgs: Vec<&str> = content.split("\nFrom ").collect();
        if msgs.is_empty() || (msgs.len() == 1 && msgs[0].is_empty()) {
            eprintln!("No mail for {}", user);
            std::process::exit(1);
        }
        for (i, msg) in msgs.iter().enumerate() {
            if msg.is_empty() { continue; }
            let first_line = msg.lines().next().unwrap_or("");
            let subject = msg.lines()
                .find(|l| l.to_lowercase().starts_with("subject:"))
                .map(|l| l.trim_start_matches("Subject:").trim_start_matches("subject:").trim())
                .unwrap_or("(no subject)");
            let from = msg.lines()
                .find(|l| l.to_lowercase().starts_with("from:"))
                .map(|l| l.trim_start_matches("From:").trim_start_matches("from:").trim())
                .unwrap_or(first_line);
            println!("{} {:<20} {}", i + 1, from, subject);
        }
        // Show first message if user hits enter with a number
        return;
    }

    match args[1].as_str() {
        "-s" => {
            // Send mode: mailx -s subject recipient
            if args.len() < 4 {
                eprintln!("Usage: mailx -s subject recipient");
                std::process::exit(1);
            }
            let subject = &args[2];
            let recipient = &args[3];
            let mut body = String::new();
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => { body.push_str(&l); body.push('\n'); }
                    Err(_) => break,
                }
            }
            send_mail(recipient, subject, &body, &user);
        }
        "-f" => {
            // Read from specific mailbox file
            if args.len() < 3 { eprintln!("Usage: mailx -f file"); return; }
            let content = fs::read_to_string(&args[2]).unwrap_or_default();
            println!("{}", content);
        }
        n if n.parse::<usize>().is_ok() => {
            // Show specific message number
            let content = fs::read_to_string(&mbox).unwrap_or_default();
            let msgs: Vec<&str> = content.split("\nFrom ").collect();
            let idx: usize = n.parse().unwrap();
            if idx > 0 && idx <= msgs.len() {
                if idx == 1 {
                    println!("{}", msgs[0]);
                } else {
                    println!("From {}", msgs[idx - 1]);
                }
            }
        }
        recipient => {
            // Simple send: mailx recipient < body (or read stdin after prompt)
            let mut body = String::new();
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => { body.push_str(&l); body.push('\n'); }
                    Err(_) => break,
                }
            }
            send_mail(recipient, "", &body, &user);
        }
    }
}

fn send_mail(recipient: &str, subject: &str, body: &str, user: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let maildir = format!("{}/mail", home);
    let _ = fs::create_dir_all(&maildir);
    let outbox = format!("{}/sent", maildir);
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let now = chrono_now();
    let mut msg = String::new();
    msg.push_str(&format!("From {} {} {}\n", user, hostname, now));
    msg.push_str(&format!("From: {}\n", user));
    msg.push_str(&format!("To: {}\n", recipient));
    if !subject.is_empty() {
        msg.push_str(&format!("Subject: {}\n", subject));
    }
    msg.push_str(&format!("Date: {}\n", now));
    msg.push('\n');
    msg.push_str(body);
    if !body.ends_with('\n') {
        msg.push('\n');
    }
    fs::OpenOptions::new().create(true).append(true).open(&outbox)
        .and_then(|mut f| f.write_all(msg.as_bytes())).ok();
    // Also append to recipient's mbox if local
    let recip_mbox = format!("{}/mbox", home);
    fs::OpenOptions::new().create(true).append(true).open(&recip_mbox)
        .and_then(|mut f| f.write_all(msg.as_bytes())).ok();
    eprintln!("mail sent to {}", recipient);
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let s = d.as_secs();
    // Return RFC 2822-like date
    format!("{}", s)
}
