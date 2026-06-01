#![no_std]
#![no_main]

use user::*;

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const DNS_PORT: u16 = 53;
const DNS_SERVER: [u8; 4] = [10, 0, 2, 3];
const TEST_DOMAIN: &str = "example.com";
const MAX_RETRIES: usize = 100;
const MAX_RECORDS: usize = 16;

fn encode_dns_name(name: &str, buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for label in name.split('.') {
        buf[pos] = label.len() as u8;
        buf[pos + 1..pos + 1 + label.len()].copy_from_slice(label.as_bytes());
        pos += 1 + label.len();
    }
    buf[pos] = 0;
    pos + 1
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

fn decode_dns_name(data: &[u8], offset: &mut usize, out: &mut [u8]) -> Option<usize> {
    let mut pos = 0;
    let mut jumped = false;
    let mut jump_return = 0;
    loop {
        if *offset >= data.len() { return None; }
        let len = data[*offset] as usize;
        if len == 0 {
            *offset += 1;
            if jumped { *offset = jump_return; }
            break;
        }
        if len & 0xc0 == 0xc0 {
            let ptr = ((len & 0x3f) << 8) | data[*offset + 1] as usize;
            if !jumped { jump_return = *offset + 2; }
            *offset = ptr;
            jumped = true;
            continue;
        }
        *offset += 1;
        if *offset + len > data.len() { return None; }
        if pos > 0 { out[pos] = b'.'; pos += 1; }
        out[pos..pos + len].copy_from_slice(&data[*offset..*offset + len]);
        pos += len;
        *offset += len;
    }
    Some(pos)
}

fn parse_a_records(data: &[u8], query_id: u16, ips: &mut [[u8; 4]]) -> usize {
    if data.len() < 12 { return 0; }
    let id = u16::from_be_bytes([data[0], data[1]]);
    if id != query_id { return 0; }
    let flags = u16::from_be_bytes([data[2], data[3]]);
    if flags & 0x000f != 0 { return 0; }
    let ancount = u16::from_be_bytes([data[6], data[7]]);
    if ancount == 0 { return 0; }
    let mut offset = 12;
    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let mut name_buf = [0u8; 256];
    for _ in 0..qdcount {
        if decode_dns_name(data, &mut offset, &mut name_buf).is_none() { return 0; }
        offset += 4;
    }
    let mut count = 0;
    for _ in 0..ancount {
        if decode_dns_name(data, &mut offset, &mut name_buf).is_none() { return 0; }
        if offset + 10 > data.len() { return 0; }
        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2; offset += 2; offset += 4;
        let rdlength = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        if offset + rdlength as usize > data.len() { return 0; }
        if rtype == TYPE_A && rdlength == 4 && count < MAX_RECORDS {
            ips[count] = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            count += 1;
        }
        offset += rdlength as usize;
    }
    count
}

fn wait_for_dhcp(fd: Fd) {
    let gw = [10, 0, 2, 2];
    let payload = b"dhcp probe";
    for _ in 0..MAX_RETRIES {
        match send(fd, payload, &gw, 9999) {
            Ok(_) => return,
            Err(e) => {
                assert_eq!(e, SysError::NoEntry);
                let _ = sleep(5);
            }
        }
    }
    panic!("DHCP did not complete");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    let fd = socket(0).expect("socket open");
    wait_for_dhcp(fd);

    let id = 0x1234u16;
    let mut query_buf = [0u8; 512];
    let qlen = build_query(TEST_DOMAIN, id, &mut query_buf);

    let n = send(fd, &query_buf[..qlen], &DNS_SERVER, DNS_PORT).expect("dns send");
    assert!(n > 0, "dns send must return > 0");

    let mut buf = [0u8; 512];
    let mut src_ip = [0u8; 4];
    let mut src_port = 0u16;
    let mut ips = [[0u8; 4]; MAX_RECORDS];

    let mut received = false;
    for _ in 0..50 {
        if let Ok(n) = receive(fd, &mut buf, &mut src_ip, &mut src_port) {
            let count = parse_a_records(&buf[..n], id, &mut ips);
            assert!(count > 0, "expected at least one A record for {}", TEST_DOMAIN);
            assert_eq!(src_ip, DNS_SERVER, "response must come from DNS server");
            assert_eq!(src_port, DNS_PORT, "response must come from port 53");
            received = true;
            break;
        }
        let _ = sleep(5);
    }
    assert!(received, "DNS query timed out");

    close(fd).expect("close");
}
