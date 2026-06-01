pub const TYPE_A: u16 = 1;
pub const TYPE_AAAA: u16 = 28;
pub const CLASS_IN: u16 = 1;
pub const DEFAULT_PORT: u16 = 53;

pub const MAX_NAME_LEN: usize = 256;
pub const MAX_RECORDS: usize = 16;
pub const MAX_QUERY_SIZE: usize = 512;
pub const MAX_RESPONSE_SIZE: usize = 512;

pub fn encode_dns_name(name: &str, buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        if pos + 1 + label.len() > buf.len() {
            break;
        }
        buf[pos] = label.len() as u8;
        buf[pos + 1..pos + 1 + label.len()].copy_from_slice(label.as_bytes());
        pos += 1 + label.len();
    }
    if pos < buf.len() {
        buf[pos] = 0;
        pos += 1;
    }
    pos
}

pub fn decode_dns_name(data: &[u8], offset: &mut usize, out: &mut [u8]) -> Option<usize> {
    let mut pos = 0;
    let mut jumped = false;
    let mut jump_return = 0;

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
            if *offset + 1 >= data.len() {
                return None;
            }
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
        if pos > 0 {
            if pos >= out.len() {
                return None;
            }
            out[pos] = b'.';
            pos += 1;
        }
        if pos + len > out.len() {
            return None;
        }
        out[pos..pos + len].copy_from_slice(&data[*offset..*offset + len]);
        pos += len;
        *offset += len;
    }

    Some(pos)
}

pub fn build_query(domain: &str, qtype: u16, id: u16, buf: &mut [u8]) -> usize {
    let mut pos = 0;

    buf[pos..pos + 2].copy_from_slice(&id.to_be_bytes());
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&[0x01, 0x00]);
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&[0x00, 0x01]);
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&[0x00, 0x00]);
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&[0x00, 0x00]);
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&[0x00, 0x00]);
    pos += 2;

    pos += encode_dns_name(domain, &mut buf[pos..]);
    buf[pos..pos + 2].copy_from_slice(&qtype.to_be_bytes());
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&CLASS_IN.to_be_bytes());
    pos += 2;

    pos
}

pub struct DnsResponse {
    pub id: u16,
    pub flags: u16,
    pub ancount: u16,
    pub a_records: [[u8; 4]; MAX_RECORDS],
    pub a_count: usize,
}

pub fn parse_a_records(data: &[u8], query_id: u16) -> Option<DnsResponse> {
    if data.len() < 12 {
        return None;
    }

    let id = u16::from_be_bytes([data[0], data[1]]);
    if id != query_id {
        return None;
    }
    let flags = u16::from_be_bytes([data[2], data[3]]);
    if flags & 0x000f != 0 {
        return None;
    }
    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let ancount = u16::from_be_bytes([data[6], data[7]]);
    if ancount == 0 {
        return Some(DnsResponse {
            id,
            flags,
            ancount,
            a_records: [[0u8; 4]; MAX_RECORDS],
            a_count: 0,
        });
    }

    let mut offset = 12;
    let mut name_buf = [0u8; MAX_NAME_LEN];
    for _ in 0..qdcount {
        decode_dns_name(data, &mut offset, &mut name_buf)?;
        offset += 4;
    }

    let mut a_records = [[0u8; 4]; MAX_RECORDS];
    let mut a_count = 0;
    for _ in 0..ancount {
        decode_dns_name(data, &mut offset, &mut name_buf)?;
        if offset + 10 > data.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        offset += 2;
        offset += 4;
        let rdlength = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        if offset + rdlength as usize > data.len() {
            return None;
        }
        if rtype == TYPE_A && rdlength == 4 && a_count < MAX_RECORDS {
            a_records[a_count] = [
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ];
            a_count += 1;
        }
        offset += rdlength as usize;
    }

    Some(DnsResponse {
        id,
        flags,
        ancount,
        a_records,
        a_count,
    })
}
