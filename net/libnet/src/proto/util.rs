pub fn internet_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

pub fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let mut ip = [0u8; 4];
    let mut i = 0;
    for part in s.split('.') {
        if i >= 4 {
            return None;
        }
        ip[i] = part.parse().ok()?;
        i += 1;
    }
    if i != 4 {
        return None;
    }
    Some(ip)
}

pub fn format_ipv4(ip: &[u8; 4], buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for (i, &octet) in ip.iter().enumerate() {
        if i > 0 {
            if pos < buf.len() {
                buf[pos] = b'.';
                pos += 1;
            }
        }
        pos += write_u8_to_buf(octet, &mut buf[pos..]);
    }
    pos
}

fn write_u8_to_buf(mut n: u8, buf: &mut [u8]) -> usize {
    if n == 0 {
        if !buf.is_empty() {
            buf[0] = b'0';
        }
        return 1;
    }
    let mut digits = [0u8; 3];
    let mut dpos = 0;
    while n > 0 {
        digits[2 - dpos] = b'0' + (n % 10);
        n /= 10;
        dpos += 1;
    }
    let start = 3 - dpos;
    for (i, &b) in digits[start..].iter().enumerate() {
        if i < buf.len() {
            buf[i] = b;
        }
    }
    dpos
}

pub const fn dns_port() -> u16 {
    53
}
