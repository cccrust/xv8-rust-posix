use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "." };
    
    writeln!(std::io::stderr(), "DEBUG: Opening '{}'", path).unwrap();
    
    let entries = match fs::read_dir(path) {
        Ok(r) => r.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(e) => {
            writeln!(std::io::stderr(), "DEBUG: read_dir error: {}", e).unwrap();
            return;
        }
    };
    
    writeln!(std::io::stderr(), "DEBUG: Got {} entries", entries.len()).unwrap();
    
    let mut names: Vec<String> = Vec::new();
    for entry in entries.iter().take(20) {
        let name = entry.file_name();
        let name_str = String::from_utf8_lossy(name.as_encoded_bytes()).to_string();
        writeln!(std::io::stderr(), "DEBUG: entry '{}'", name_str).unwrap();
        names.push(name_str);
    }
    
    writeln!(std::io::stderr(), "DEBUG: First 20 names: {:?}", names).unwrap();
}
