#[cfg(feature = "xv8")]
use alloc::format;
#[cfg(feature = "xv8")]
use alloc::string::{String, ToString};
#[cfg(feature = "xv8")]
use alloc::vec::Vec;

use crate::util::encode_dns_name;

pub const TYPE_A: u16 = 1;
pub const TYPE_AAAA: u16 = 28;
pub const CLASS_IN: u16 = 1;

#[derive(Debug)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&self.qdcount.to_be_bytes());
        buf.extend_from_slice(&self.ancount.to_be_bytes());
        buf.extend_from_slice(&self.nscount.to_be_bytes());
        buf.extend_from_slice(&self.arcount.to_be_bytes());
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        Some(DnsHeader {
            id: u16::from_be_bytes([data[0], data[1]]),
            flags: u16::from_be_bytes([data[2], data[3]]),
            qdcount: u16::from_be_bytes([data[4], data[5]]),
            ancount: u16::from_be_bytes([data[6], data[7]]),
            nscount: u16::from_be_bytes([data[8], data[9]]),
            arcount: u16::from_be_bytes([data[10], data[11]]),
        })
    }

    pub fn is_success(&self) -> bool {
        self.flags & 0x000f == 0
    }
}

#[derive(Debug)]
pub struct DnsQuestion {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl DnsQuestion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = encode_dns_name(&self.qname);
        buf.extend_from_slice(&self.qtype.to_be_bytes());
        buf.extend_from_slice(&self.qclass.to_be_bytes());
        buf
    }
}

#[derive(Debug)]
pub struct DnsRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl DnsRecord {
    pub fn to_ipv4(&self) -> Option<[u8; 4]> {
        if self.rtype == TYPE_A && self.rdlength == 4 {
            Some([self.rdata[0], self.rdata[1], self.rdata[2], self.rdata[3]])
        } else {
            None
        }
    }

    pub fn to_ipv6(&self) -> Option<[u8; 16]> {
        if self.rtype == TYPE_AAAA && self.rdlength == 16 {
            let mut addr = [0u8; 16];
            addr.copy_from_slice(&self.rdata);
            Some(addr)
        } else {
            None
        }
    }
}

fn decode_dns_name(data: &[u8], offset: &mut usize) -> Option<String> {
    let mut labels = Vec::new();
    let mut jumped = false;
    let mut jump_return = 0;

    if *offset >= data.len() {
        return None;
    }

    loop {
        if *offset >= data.len() {
            return None;
        }
        let len = data[*offset] as usize;
        if len == 0 {
            *offset += 1;
            if jumped {
                *offset = jump_return;
            }
            break;
        }
        if len & 0xc0 == 0xc0 {
            let ptr = ((len & 0x3f) << 8) | data[*offset + 1] as usize;
            if !jumped {
                jump_return = *offset + 2;
            }
            *offset = ptr;
            jumped = true;
            continue;
        }
        *offset += 1;
        if *offset + len > data.len() {
            return None;
        }
        let label = core::str::from_utf8(&data[*offset..*offset + len]).ok()?;
        labels.push(label);
        *offset += len;
    }

    if labels.is_empty() {
        return Some(String::new());
    }
    Some(labels.join("."))
}

fn parse_record(data: &[u8], offset: &mut usize) -> Option<DnsRecord> {
    let name = decode_dns_name(data, offset)?;
    if *offset + 10 > data.len() {
        return None;
    }
    let rtype = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    let rclass = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    let ttl = u32::from_be_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    let rdlength = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    if *offset + rdlength as usize > data.len() {
        return None;
    }
    let rdata = data[*offset..*offset + rdlength as usize].to_vec();
    *offset += rdlength as usize;
    Some(DnsRecord {
        name,
        rtype,
        rclass,
        ttl,
        rdlength,
        rdata,
    })
}

pub fn build_query(domain: &str, qtype: u16, id: u16) -> Vec<u8> {
    let header = DnsHeader {
        id,
        flags: 0x0100,
        qdcount: 1,
        ancount: 0,
        nscount: 0,
        arcount: 0,
    };
    let question = DnsQuestion {
        qname: domain.to_string(),
        qtype,
        qclass: CLASS_IN,
    };
    let mut buf = header.to_bytes();
    buf.extend_from_slice(&question.to_bytes());
    buf
}

pub fn parse_response(data: &[u8]) -> Option<(DnsHeader, Vec<DnsRecord>)> {
    let header = DnsHeader::from_bytes(data)?;
    if header.ancount == 0 {
        return Some((header, Vec::new()));
    }
    let mut offset = 12;
    for _ in 0..header.qdcount {
        decode_dns_name(data, &mut offset)?;
        offset += 4;
    }
    let mut records = Vec::new();
    for _ in 0..header.ancount {
        records.push(parse_record(data, &mut offset)?);
    }
    Some((header, records))
}

/// Sends a DNS query over UDP and returns the (header, records) response.
pub fn query(server: &str, domain: &str, qtype: u16) -> Result<(DnsHeader, Vec<DnsRecord>), String> {
    use crate::net_impl::{UdpSocket, Duration};

    let sock = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("bind: {}", e))?;
    sock.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("set timeout: {}", e))?;

    let id = crate::random_u16();
    let query = build_query(domain, qtype, id);
    sock.send_to(&query, format!("{}:{}", server, 53))
        .map_err(|e| format!("send: {}", e))?;

    let mut buf = [0u8; 512];
    let (n, _) = sock
        .recv_from(&mut buf)
        .map_err(|e| format!("recv: {}", e))?;

    let (header, records) =
        parse_response(&buf[..n]).ok_or_else(|| "parse failed".to_string())?;
    if !header.is_success() {
        return Err(format!("DNS error: flags={:#06x}", header.flags));
    }
    Ok((header, records))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse_query() {
        let q = build_query("google.com", TYPE_A, 42);
        assert!(q.len() > 12);
        let h = DnsHeader::from_bytes(&q).unwrap();
        assert_eq!(h.id, 42);
        assert_eq!(h.qdcount, 1);
    }

    #[test]
    fn test_dns_name_roundtrip() {
        let mut offset = 0;
        let data = encode_dns_name("www.example.com");
        let name = decode_dns_name(&data, &mut offset).unwrap();
        assert_eq!(name, "www.example.com");
    }

    #[test]
    fn test_dns_name_with_pointer() {
        // Simulate: first encode "example.com" at offset 0, then place a pointer at offset 20
        let mut data = encode_dns_name("example.com");
        // Pad to offset 22
        while data.len() < 22 {
            data.push(0);
        }
        // Pointer to offset 0: 0xc0 0x00
        data[20] = 0xc0;
        data[21] = 0x00;
        let mut offset = 20;
        let name = decode_dns_name(&data, &mut offset).unwrap();
        assert_eq!(name, "example.com");
        // offset should be 22 (past the pointer)
        assert_eq!(offset, 22);
    }
}
