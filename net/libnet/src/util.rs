#[cfg(feature = "xv8")]
use alloc::{format, vec};
#[cfg(feature = "xv8")]
use alloc::string::String;
#[cfg(feature = "xv8")]
use alloc::vec::Vec;

pub fn ip_to_string(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

pub fn string_to_ip(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut ip = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        ip[i] = p.parse().ok()?;
    }
    Some(ip)
}

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

pub fn encode_dns_name(name: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        result.push(label.len() as u8);
        result.extend_from_slice(label.as_bytes());
    }
    result.push(0);
    result
}

pub const fn dns_port() -> u16 {
    53
}

pub struct Ipv4Header {
    pub src: [u8; 4],
    pub dst: [u8; 4],
    pub protocol: u8,
    pub total_len: u16,
}

pub fn build_ipv4_header(iph: &Ipv4Header, payload: &[u8]) -> Vec<u8> {
    let total_len = (20 + payload.len()) as u16;
    let mut header = vec![0u8; 20];
    header[0] = 0x45;
    header[2..4].copy_from_slice(&total_len.to_be_bytes());
    header[8] = 64;
    header[9] = iph.protocol;
    header[12..16].copy_from_slice(&iph.src);
    header[16..20].copy_from_slice(&iph.dst);
    let checksum = internet_checksum(&header);
    header[10..12].copy_from_slice(&checksum.to_be_bytes());
    header.extend_from_slice(payload);
    header
}

pub fn fmt_duration_us(secs: f64) -> String {
    if secs < 1.0 {
        format!("{:.3} ms", secs * 1000.0)
    } else {
        format!("{:.3} s", secs)
    }
}
