use std::time::Duration;
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: sleep seconds");
        std::process::exit(1);
    }

    let total_secs: f64 = args[1..].iter()
        .filter_map(|s| s.parse::<f64>().ok())
        .sum();

    let secs = total_secs as u64;
    let nanos = ((total_secs - secs as f64) * 1_000_000_000.0) as u32;

    thread::sleep(Duration::new(secs, nanos));
}
