fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: gettext [-d domain] msgid");
        std::process::exit(1);
    }
    let mut domain = std::env::var("TEXTDOMAIN").unwrap_or_default();
    let mut msgid = String::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-d" => {
                i += 1;
                domain = args[i].clone();
            }
            _ => {
                msgid = args[i].clone();
            }
        }
        i += 1;
    }
    if msgid.is_empty() {
        return;
    }
    let localedir = std::env::var("TEXTDOMAINDIR").unwrap_or_else(|_| "/usr/share/locale".to_string());
    let lang = std::env::var("LANG").unwrap_or_else(|_| "C".to_string());
    let lang = lang.split('.').next().unwrap_or(&lang);
    let mo_path = format!("{}/{}/LC_MESSAGES/{}.mo", localedir, lang, domain);
    if let Ok(data) = std::fs::read(&mo_path) {
        if let Some(trans) = lookup_mo(&data, &msgid) {
            println!("{}", trans);
            return;
        }
    }
    println!("{}", msgid);
}

fn lookup_mo(data: &[u8], msgid: &str) -> Option<String> {
    if data.len() < 28 {
        return None;
    }
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != 0x950412DE {
        return None;
    }
    let n = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let orig_off = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let trans_off = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    for i in 0..n {
        let off = (orig_off + i * 8) as usize;
        if off + 8 > data.len() {
            break;
        }
        let len = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        let str_off = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
        let str_off = str_off as usize;
        if str_off + len as usize > data.len() {
            continue;
        }
        let id = std::str::from_utf8(&data[str_off..str_off + len as usize]).ok()?;
        if id == msgid {
            let trans_off2 = (trans_off + i * 8) as usize;
            if trans_off2 + 8 > data.len() {
                break;
            }
            let tlen = u32::from_le_bytes([data[trans_off2], data[trans_off2+1], data[trans_off2+2], data[trans_off2+3]]);
            let tstr_off = u32::from_le_bytes([data[trans_off2+4], data[trans_off2+5], data[trans_off2+6], data[trans_off2+7]]);
            let tstr_off = tstr_off as usize;
            if tstr_off + tlen as usize > data.len() {
                break;
            }
            let trans = std::str::from_utf8(&data[tstr_off..tstr_off + tlen as usize]).ok()?;
            return Some(trans.to_string());
        }
    }
    None
}
