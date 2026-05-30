use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;

    while i < args.len() && args[i].starts_with('-') && args[i] != "--" {
        if args[i] == "--" { i += 1; break; }
        eprintln!("od: invalid option -- '{}'", args[i]);
        std::process::exit(1);
    }

    let files: Vec<String> = args[i..].to_vec();
    let mut data = Vec::new();

    if files.is_empty() {
        let _ = std::io::stdin().lock().read_to_end(&mut data);
    } else {
        let mut f = File::open(Path::new(&files[0])).unwrap_or_else(|e| {
            eprintln!("od: {}: {}", files[0], e);
            std::process::exit(1);
        });
        let _ = f.read_to_end(&mut data);
    }

    let bytes_per_group = 2;
    let groups_per_line = 8;

    for (chunk_idx, chunk) in data.chunks(bytes_per_group * groups_per_line).enumerate() {
        let addr = chunk_idx * bytes_per_group * groups_per_line;
        print!("{:07o}", addr);

        for group in chunk.chunks(bytes_per_group) {
            let val = if group.len() >= 2 {
                (group[0] as u16) | ((group[1] as u16) << 8)
            } else {
                group[0] as u16
            };
            print!(" {:06o}", val);
        }
        println!();
    }

    let total = data.len();
    if total > 0 {
        println!("{:07o}", total);
    }
}
