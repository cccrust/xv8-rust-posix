use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: msgfmt [-o output] file.po");
        std::process::exit(1);
    }
    let mut output = "messages.mo";
    let mut input = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                output = &args[i];
            }
            _ => {
                input = Some(&args[i]);
            }
        }
        i += 1;
    }
    let input = match input {
        Some(p) => p,
        None => {
            eprintln!("msgfmt: missing input file");
            std::process::exit(1);
        }
    };
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("msgfmt: {}: {}", input, e);
            std::process::exit(1);
        }
    };
    let mut translations: Vec<(String, String)> = Vec::new();
    let mut msgid = String::new();
    let mut msgstr = String::new();
    let mut in_id = false;
    let mut in_str = false;
    for line in content.lines() {
        if line.starts_with("msgid") {
            if in_str && !msgid.is_empty() {
                translations.push((msgid.clone(), msgstr.clone()));
            }
            msgid.clear();
            msgstr.clear();
            in_id = true;
            in_str = false;
            if let Some(val) = line.strip_prefix("msgid \"") {
                if let Some(v) = val.strip_suffix('"') {
                    msgid = v.to_string();
                    in_id = false;
                }
            }
        } else if line.starts_with("msgstr") {
            in_id = false;
            in_str = true;
            if let Some(val) = line.strip_prefix("msgstr \"") {
                if let Some(v) = val.strip_suffix('"') {
                    msgstr = v.to_string();
                }
            }
        } else if line.starts_with("msgstr[") {
            in_str = true;
        } else if in_id && line.starts_with('"') {
            if let Some(v) = line.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                msgid.push_str(v);
            }
        } else if in_str && line.starts_with('"') {
            if let Some(v) = line.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                msgstr.push_str(v);
            }
        }
    }
    if in_str && !msgid.is_empty() {
        translations.push((msgid, msgstr));
    }
    // Write minimal .mo format (GNU gettext)
    let mut mo = Vec::new();
    // Magic
    mo.extend_from_slice(&[0xDE, 0x12, 0x04, 0x95]); // little-endian magic
    // Version
    mo.extend_from_slice(&0u32.to_le_bytes());
    // Number of strings
    let n = translations.len() as u32;
    mo.extend_from_slice(&n.to_le_bytes());
    // Offset of original table
    let orig_table_off: u32 = 28;
    mo.extend_from_slice(&orig_table_off.to_le_bytes());
    // Offset of translation table
    let mut total_len: u32 = 0;
    for (id, _) in &translations {
        total_len += id.len() as u32 + 1;
    }
    let trans_table_off = orig_table_off + n * 8;
    mo.extend_from_slice(&trans_table_off.to_le_bytes());
    // Original table
    let mut offsets_orig = Vec::new();
    let mut offsets_trans = Vec::new();
    for (id, str) in &translations {
        offsets_orig.push((total_len, id.len() as u32));
        total_len += id.len() as u32 + 1;
        offsets_trans.push((total_len, str.len() as u32));
        total_len += str.len() as u32 + 1;
    }
    for (off, len) in &offsets_orig {
        mo.extend_from_slice(&len.to_le_bytes());
        mo.extend_from_slice(&off.to_le_bytes());
    }
    for (off, len) in &offsets_trans {
        mo.extend_from_slice(&len.to_le_bytes());
        mo.extend_from_slice(&off.to_le_bytes());
    }
    for (id, str) in &translations {
        mo.extend_from_slice(id.as_bytes());
        mo.push(0);
        mo.extend_from_slice(str.as_bytes());
        mo.push(0);
    }
    if let Err(e) = fs::write(output, &mo) {
        eprintln!("msgfmt: {}: {}", output, e);
        std::process::exit(1);
    }
}
