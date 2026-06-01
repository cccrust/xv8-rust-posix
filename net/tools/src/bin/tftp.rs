use libnet::tftp;
use std::fs;

fn usage() -> ! {
    eprintln!("Usage: tftp <host> get <remote-file> [local-file]");
    eprintln!("       tftp <host> put <local-file> [remote-file]");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        usage();
    }

    let host = &args[1];
    let cmd = &args[2];
    match cmd.as_str() {
        "get" => {
            let remote = &args[3];
            let local = args.get(4).cloned().unwrap_or_else(|| remote.clone());
            eprintln!("Downloading {} from {}...", remote, host);
            match tftp::download(host, remote) {
                Ok(data) => {
                    fs::write(&local, &data).unwrap_or_else(|e| {
                        eprintln!("write {}: {}", local, e);
                        std::process::exit(1);
                    });
                    eprintln!("Saved {} bytes to {}", data.len(), local);
                }
                Err(e) => {
                    eprintln!("tftp: {}: {}", remote, e);
                    std::process::exit(1);
                }
            }
        }
        "put" => {
            let local = &args[3];
            let _remote = args.get(4).cloned().unwrap_or_else(|| local.clone());
            eprintln!("Uploading {} to {}... (not yet implemented)", local, host);
            std::process::exit(1);
        }
        _ => usage(),
    }
}
