use std::env;
use std::process::exit;
use ureq::{Agent, AgentBuilder};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <URL>", args[0]);
        exit(1);
    }

    let url = &args[1];
    let agent = AgentBuilder::new()
        .timeout_read(std::time::Duration::from_secs(10))
        .timeout_write(std::time::Duration::from_secs(10))
        .build();

    match agent.get(url).call() {
        Ok(response) => {
            println!("Status: {}", response.status());
            match response.into_string() {
                Ok(body) => {
                    print!("{}", body);
                }
                Err(e) => {
                    eprintln!("Failed to read response body: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch URL: {}", e);
            exit(1);
        }
    }
}