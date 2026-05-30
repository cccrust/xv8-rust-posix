use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let mut rflag = false;
    if !files.is_empty() && files[0] == "-r" {
        rflag = true;
        files.remove(0);
    }
    if files.is_empty() {
        files.push("-");
    }
    for f in files {
        let data: Vec<u8> = if f == "-" {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap_or_default();
            buf
        } else {
            std::fs::read(f).unwrap_or_default()
        };
        let len = data.len();
        if rflag {
            let s = bsd_sum(&data);
            println!("{} {} {}", s as u32, len, if f == "-" { "" } else { f });
        } else {
            let s = sysv_sum(&data);
            println!("{} {} {}", s, len, if f == "-" { "" } else { f });
        }
    }
}

fn sysv_sum(data: &[u8]) -> u32 {
    let mut s: u32 = 0;
    for &b in data {
        s = s.wrapping_add(b as u32);
    }
    (s & 0xffff) + (s >> 16)
}

fn bsd_sum(data: &[u8]) -> u16 {
    let mut s: u16 = 0;
    for &b in data {
        let carry = s & 0x8000;
        s = (s << 1) | (carry >> 15);
        s = s.wrapping_add(b as u16);
    }
    s
}
