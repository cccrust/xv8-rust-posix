#[cfg(not(target_arch = "riscv64"))]
use std::os::unix::fs::MetadataExt;

fn main() {
    // Use std::fs just like test_ls
    use std::fs;
    
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() < 2 { "." } else { &args[1] };
    
    let mut entries: Vec<_> = match fs::read_dir(path) {
        Ok(r) => r.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            eprintln!("cannot open {}: {}", path, e);
            return;
        }
    };
    
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    
    let total: u64 = entries.iter().filter_map(|e| e.metadata().ok()).map(|m| m.blocks()).sum();
    println!("total {}", total);
    
    for entry in entries.iter().take(10) {
        let name = entry.file_name();
        let name_str = String::from_utf8_lossy(name.as_encoded_bytes()).to_string();
        println!("{}", name_str);
    }
}
