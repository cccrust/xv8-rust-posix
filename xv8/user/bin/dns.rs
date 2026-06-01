#![no_std]
#![no_main]

use user::*;

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const DNS_PORT: u16 = 53;
const TIMEOUT_TICKS: usize = 50;
const MAX_RECORDS: usize = 16;

fn encode_dns_name(name: &str, buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for label in name.split('.') {
        if pos + 1 + label.len() > 255 {
            break;
        }
        buf[pos] = label.len() as u8;
        buf[pos + 1..pos + 1 + label.len()].copy_from_slice(label.as_bytes());
        pos += 1 + label.len();
    }
    buf[pos] = 0;
    pos + 1
}

fn decode_dns_name(data: &[u8], offset: &mut usize, out: &mut [u8]) -> Option<usize> {
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

fn build_query(domain: &str, id: u16, buf: &mut [u8]) -> usize {
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
    buf[pos..pos + 2].copy_from_slice(&TYPE_A.to_be_bytes());
    pos += 2;
    buf[pos..pos + 2].copy_from_slice(&CLASS_IN.to_be_bytes());
    pos += 2;
    pos
}

fn parse_ip_from_response(data: &[u8], query_id: u16, ips: &mut [[u8; 4]], max_ips: &mut usize) -> Result<usize, &'static str> {
    if data.len() < 12 {
        return Err("response too short");
    }
    let id = u16::from_be_bytes([data[0], data[1]]);
    if id != query_id {
        return Err("ID mismatch");
    }
    let flags = u16::from_be_bytes([data[2], data[3]]);
    if flags & 0x000f != 0 {
        return Err("DNS error");
    }
    let ancount = u16::from_be_bytes([data[6], data[7]]);
    if ancount == 0 {
        return Ok(0);
    }
    let mut offset = 12;
    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let mut name_buf = [0u8; 256];
    for _ in 0..qdcount {
        decode_dns_name(data, &mut offset, &mut name_buf).ok_or("bad question name")?;
        offset += 4;
    }
    let mut count = 0;
    let limit = (*max_ips).min(ancount as usize);
    for _ in 0..ancount {
        decode_dns_name(data, &mut offset, &mut name_buf).ok_or("bad record name")?;
        if offset + 10 > data.len() {
            return Err("record header truncated");
        }
        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        offset += 2; // rclass
        offset += 4; // ttl
        let rdlength = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        if offset + rdlength as usize > data.len() {
            return Err("rdata truncated");
        }
        if rtype == TYPE_A && rdlength == 4 && count < limit {
            ips[count] = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            count += 1;
        }
        offset += rdlength as usize;
    }
    *max_ips = count;
    Ok(count)
}

#[unsafe(no_mangle)]
fn main(args: Args) {
    if args.len() < 3 {
        eprintln!("usage: dns <server> <domain>");
        exit(1);
    }
    let server = args.get_str(1).unwrap();
    let domain = args.get_str(2).unwrap();

    let server_ip = match server.parse::<Ipv4Addr>() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("invalid server IP: {}", server);
            exit(1);
        }
    };

    let fd = socket(0).unwrap_or_else(|_| {
        eprintln!("socket failed");
        exit(1);
    });

    let id = (uptime() & 0xffff) as u16;
    let mut query_buf = [0u8; 512];
    let qlen = build_query(domain, id, &mut query_buf);

    send(fd, &query_buf[..qlen], &server_ip.0, DNS_PORT).unwrap_or_else(|_| {
        eprintln!("send failed");
        exit(1);
    });

    let mut buf = [0u8; 512];
    let mut src_ip = [0u8; 4];
    let mut src_port = 0u16;
    let n = receive_timeout(fd, &mut buf, &mut src_ip, &mut src_port, TIMEOUT_TICKS)
        .unwrap_or_else(|_| {
            eprintln!("receive timeout");
            exit(1);
        });

    let mut ips = [[0u8; 4]; MAX_RECORDS];
    let mut max_ips = MAX_RECORDS;
    match parse_ip_from_response(&buf[..n], id, &mut ips, &mut max_ips) {
        Ok(count) => {
            for i in 0..count {
                println!("{}", Ipv4Addr(ips[i]));
            }
            if count == 0 {
                eprintln!("no A records found");
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("parse error: {}", e);
            exit(1);
        }
    }
}

fn receive_timeout(
    fd: Fd,
    buf: &mut [u8],
    src_ip: &mut [u8; 4],
    src_port: &mut u16,
    ticks: usize,
) -> Result<usize, SysError> {
    let step = 5;
    let mut waited = 0;
    while waited < ticks {
        if let Ok(n) = receive(fd, buf, src_ip, src_port) {
            return Ok(n);
        }
        sleep(step)?;
        waited += step;
    }
    Err(SysError::NoEntry)
}
