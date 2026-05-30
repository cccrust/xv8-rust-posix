use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: strip [-o output] file...");
        std::process::exit(1);
    }
    let mut files: Vec<&str> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => { i += 2; }
            "-s" | "-S" | "--strip-all" | "-x" | "-X" | "-g" => { i += 1; }
            _ => {
                files.push(&args[i]);
                i += 1;
            }
        }
    }
    for fname in files {
        if let Err(e) = strip_file(fname) {
            if e.kind() == std::io::ErrorKind::InvalidData {
                eprintln!("strip: {}: file format not recognized", fname);
            } else {
                eprintln!("strip: {}: {}", fname, e);
            }
            std::process::exit(1);
        }
    }
}

fn strip_file(path: &str) -> std::io::Result<()> {
    let mut data = fs::read(path)?;
    if data.len() < 64 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "too small"));
    }
    // Only handle ELF
    if !data.starts_with(b"\x7fELF") {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not ELF"));
    }
    let is_64 = data[4] == 2;
    let (shoff, shentsize, shnum, shstrndx) = if is_64 {
        let shoff = u64::from_le_bytes([data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47]]);
        let shentsize = data[58] as u16;
        let shnum = u16::from_le_bytes([data[60], data[61]]);
        let shstrndx = u16::from_le_bytes([data[62], data[63]]);
        (shoff as usize, shentsize, shnum, shstrndx)
    } else {
        let shoff = u32::from_le_bytes([data[32], data[33], data[34], data[35]]) as usize;
        let shentsize = data[46] as u16;
        let shnum = u16::from_le_bytes([data[48], data[49]]);
        let shstrndx = u16::from_le_bytes([data[50], data[51]]);
        (shoff, shentsize, shnum, shstrndx)
    };
    // Read section header string table
    let shstrtab_off = shoff + shstrndx as usize * shentsize as usize;
    let (shstrtab_sec_off, shstrtab_sec_size) = if is_64 {
        let off = u64::from_le_bytes([
            data[shstrtab_off+24], data[shstrtab_off+25], data[shstrtab_off+26], data[shstrtab_off+27],
            data[shstrtab_off+28], data[shstrtab_off+29], data[shstrtab_off+30], data[shstrtab_off+31],
        ]);
        let size = u64::from_le_bytes([
            data[shstrtab_off+32], data[shstrtab_off+33], data[shstrtab_off+34], data[shstrtab_off+35],
            data[shstrtab_off+36], data[shstrtab_off+37], data[shstrtab_off+38], data[shstrtab_off+39],
        ]);
        (off as usize, size as usize)
    } else {
        let off = u32::from_le_bytes([data[shstrtab_off+16], data[shstrtab_off+17], data[shstrtab_off+18], data[shstrtab_off+19]]);
        let size = u32::from_le_bytes([data[shstrtab_off+20], data[shstrtab_off+21], data[shstrtab_off+22], data[shstrtab_off+23]]);
        (off as usize, size as usize)
    };
    let strtab = data[shstrtab_sec_off..shstrtab_sec_off+shstrtab_sec_size].to_vec();

    // Find .symtab and .strtab sections
    let mut symtab_ndx = None;
    let mut strtab_ndx = None;
    for ndx in 0..shnum {
        let sh_off = shoff + ndx as usize * shentsize as usize;
        let name_off = if is_64 {
            u32::from_le_bytes([data[sh_off], data[sh_off+1], data[sh_off+2], data[sh_off+3]])
        } else {
            u32::from_le_bytes([data[sh_off], data[sh_off+1], data[sh_off+2], data[sh_off+3]])
        } as usize;
        if name_off < strtab.len() {
            let name_end = strtab[name_off..].iter().position(|&b| b == 0).unwrap_or(strtab.len() - name_off);
            let name = String::from_utf8_lossy(&strtab[name_off..name_off+name_end]);
            if name == ".symtab" { symtab_ndx = Some(ndx); }
            if name == ".strtab" { strtab_ndx = Some(ndx); }
        }
    }

    // Zero out .symtab and .strtab section headers and their data
    if let Some(symtab_ndx) = symtab_ndx {
        let sh_off = shoff + symtab_ndx as usize * shentsize as usize;
        let (sec_off, sec_size) = get_section_info(&data, sh_off, is_64);
        // Zero the actual section data in the file
        for j in sec_off..sec_off+sec_size {
            if j < data.len() { data[j] = 0; }
        }
        // Zero the section header
        for j in sh_off..sh_off+shentsize as usize {
            if j < data.len() { data[j] = 0; }
        }
    }
    if let Some(strtab_ndx) = strtab_ndx {
        let sh_off = shoff + strtab_ndx as usize * shentsize as usize;
        let (sec_off, sec_size) = get_section_info(&data, sh_off, is_64);
        for j in sec_off..sec_off+sec_size {
            if j < data.len() { data[j] = 0; }
        }
        for j in sh_off..sh_off+shentsize as usize {
            if j < data.len() { data[j] = 0; }
        }
    }

    // Remove write temp, then replace
    let tmp = format!("{}.stripped", path);
    let mut f = fs::File::create(&tmp)?;
    f.write_all(&data)?;
    drop(f);
    fs::rename(&tmp, path)?;
    Ok(())
}

fn get_section_info(data: &[u8], sh_off: usize, is_64: bool) -> (usize, usize) {
    if is_64 {
        let off = u64::from_le_bytes([
            data[sh_off+24], data[sh_off+25], data[sh_off+26], data[sh_off+27],
            data[sh_off+28], data[sh_off+29], data[sh_off+30], data[sh_off+31],
        ]) as usize;
        let size = u64::from_le_bytes([
            data[sh_off+32], data[sh_off+33], data[sh_off+34], data[sh_off+35],
            data[sh_off+36], data[sh_off+37], data[sh_off+38], data[sh_off+39],
        ]) as usize;
        (off, size)
    } else {
        let off = u32::from_le_bytes([data[sh_off+16], data[sh_off+17], data[sh_off+18], data[sh_off+19]]) as usize;
        let size = u32::from_le_bytes([data[sh_off+20], data[sh_off+21], data[sh_off+22], data[sh_off+23]]) as usize;
        (off, size)
    }
}
