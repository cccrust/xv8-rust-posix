use std::fs;
use std::io::Write;

fn main() {
    let entries = match fs::read_dir(".") {
        Ok(r) => r.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("read_dir error: {}", e);
            return;
        }
    };
    
    println!("total {}", entries.len());
    
    for entry in entries.iter().take(10) {
        let name = entry.file_name();
        let name_str = String::from_utf8_lossy(name.as_encoded_bytes()).to_string();
        println!("{}", name_str);
    }
}
