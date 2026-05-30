use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<&str> = Vec::new();
    let mut defined_only = false;
    let mut extern_only = false;
    let mut undefined_only = false;
    let mut sort = false;
    let numeric = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "-B" | "-C" | "-D" | "-f" | "-h" | "-l" | "-o" | "-p" | "-r" | "-s" | "-v" => {
                i += 1;
            }
            "-c" | "--demangle" => { i += 1; }
            "-e" => { defined_only = true; i += 1; }
            "-g" | "--extern-only" => { extern_only = true; i += 1; }
            "-n" | "--numeric-sort" => { sort = true; i += 1; }
            "-u" | "--undefined-only" => { undefined_only = true; i += 1; }
            _ => {
                if args[i].starts_with('-') && args[i] != "-" {
                    eprintln!("nm: unknown option: {}", args[i]);
                    std::process::exit(1);
                }
                files.push(&args[i]);
                i += 1;
            }
        }
    }
    if files.is_empty() {
        files.push("a.out");
    }
    let nfiles = files.len();
    for fname in files {
        if nfiles > 1 {
            println!("\n{}:", fname);
        }
        if let Err(e) = nm_file(fname, defined_only, extern_only, undefined_only, sort, numeric) {
            eprintln!("nm: {}: {}", fname, e);
            std::process::exit(1);
        }
    }
}

fn nm_file(path: &str, _defined_only: bool, _extern_only: bool, _undefined_only: bool, _sort: bool, _numeric: bool) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    if !data.starts_with(b"\x7fELF") {
        return Err("file format not recognized".into());
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
    let (shstrtab_sec_off, shstrtab_sec_size) = get_section_info(&data, shstrtab_off, is_64);
    let strtab = data[shstrtab_sec_off..shstrtab_sec_off+shstrtab_sec_size].to_vec();

    // Find all section names
    let mut sec_names: Vec<String> = Vec::new();
    for ndx in 0..shnum {
        let sh_off = shoff + ndx as usize * shentsize as usize;
        let name_off = u32::from_le_bytes([data[sh_off], data[sh_off+1], data[sh_off+2], data[sh_off+3]]) as usize;
        if name_off < strtab.len() {
            let name_end = strtab[name_off..].iter().position(|&b| b == 0).unwrap_or(strtab.len() - name_off);
            sec_names.push(String::from_utf8_lossy(&strtab[name_off..name_off+name_end]).to_string());
        } else {
            sec_names.push(String::new());
        }
    }

    // Find .symtab section
    for ndx in 0..shnum {
        if sec_names[ndx as usize] == ".symtab" || sec_names[ndx as usize] == ".dynsym" {
            let sh_off = shoff + ndx as usize * shentsize as usize;
            let (sec_off, sec_size) = get_section_info(&data, sh_off, is_64);
            let entsize = if is_64 {
                u64::from_le_bytes([data[sh_off+56], data[sh_off+57], data[sh_off+58], data[sh_off+59],
                    data[sh_off+60], data[sh_off+61], data[sh_off+62], data[sh_off+63]]) as usize
            } else {
                u32::from_le_bytes([data[sh_off+36], data[sh_off+37], data[sh_off+38], data[sh_off+39]]) as usize
            };
            // Find linked strtab
            let link = if is_64 {
                u32::from_le_bytes([data[sh_off+40], data[sh_off+41], data[sh_off+42], data[sh_off+43]]) as usize
            } else {
                u32::from_le_bytes([data[sh_off+24], data[sh_off+25], data[sh_off+26], data[sh_off+27]]) as usize
            };
            let (str_off, str_size) = if link < shnum as usize && link > 0 {
                let st_sh_off = shoff + link * shentsize as usize;
                get_section_info(&data, st_sh_off, is_64)
            } else {
                (0, 0)
            };

            let sym_entsize = if entsize == 0 { if is_64 { 24 } else { 16 } } else { entsize };
            let count = sec_size / sym_entsize;

            for j in 0..count {
                let sym_off = sec_off + j * sym_entsize;
                let st_value: u64;
                let st_shndx: u16;
                let st_name: u32;

                if is_64 {
                    st_name = u32::from_le_bytes([data[sym_off], data[sym_off+1], data[sym_off+2], data[sym_off+3]]);
                    st_value = u64::from_le_bytes([
                        data[sym_off+8], data[sym_off+9], data[sym_off+10], data[sym_off+11],
                        data[sym_off+12], data[sym_off+13], data[sym_off+14], data[sym_off+15],
                    ]);
                    st_shndx = u16::from_le_bytes([data[sym_off+6], data[sym_off+7]]);
                } else {
                    st_name = u32::from_le_bytes([data[sym_off], data[sym_off+1], data[sym_off+2], data[sym_off+3]]);
                    st_value = u32::from_le_bytes([data[sym_off+4], data[sym_off+5], data[sym_off+6], data[sym_off+7]]) as u64;
                    st_shndx = u16::from_le_bytes([data[sym_off+14], data[sym_off+15]]);
                }

                // Get symbol name from string table
                let sym_name = if st_name as usize + 1 < str_size {
                    let name_end = data[str_off + st_name as usize..str_off + str_size].iter().position(|&b| b == 0).unwrap_or(0);
                    String::from_utf8_lossy(&data[str_off + st_name as usize..str_off + st_name as usize + name_end]).to_string()
                } else {
                    String::new()
                };

                if sym_name.is_empty() || sym_name.starts_with('.') {
                    continue;
                }

                let sym_info = data[sym_off + if is_64 { 4 } else { 12 }];
                let sym_other = data[sym_off + if is_64 { 5 } else { 13 }];

                let sym_type = symbol_type(st_shndx, sym_info & 0x0f, sym_other);
                let sym_bind = (sym_info >> 4) & 0x0f;

                if sym_bind == 0 { continue; } // STB_LOCAL

                if is_64 {
                    println!("{:016x} {} {}", st_value, sym_type, sym_name);
                } else {
                    println!("{:08x} {} {}", st_value as u32, sym_type, sym_name);
                }
            }
        }
    }
    Ok(())
}

fn symbol_type(shndx: u16, stt: u8, vis: u8) -> char {
    if shndx == 0 { return 'U'; }
    if shndx == 0xfff1 { return 'A'; }
    if shndx == 0xfff2 { return 'C'; }
    match stt {
        1 => { if vis == 2 { 'V' } else { 'T' } }
        2 => 'S',
        3 => 't',
        4 => 'D',
        6 => 'D',
        _ => 'D',
    }
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
