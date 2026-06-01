use crate::proto::util::internet_checksum;

pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;

pub const ICMP_HEADER_LEN: usize = 8;
pub const MAX_PING_PAYLOAD: usize = 64;
pub const DEFAULT_PING_DATA: &[u8] = b"abcdefghijklmnopqrstuvwabcdefghi";

pub fn build_echo_request(id: u16, seq: u16, payload: &[u8]) -> [u8; ICMP_HEADER_LEN + MAX_PING_PAYLOAD] {
    let mut buf = [0u8; ICMP_HEADER_LEN + MAX_PING_PAYLOAD];
    buf[0] = ICMP_ECHO_REQUEST;
    buf[1] = 0;
    buf[2..4].copy_from_slice(&0u16.to_be_bytes());
    buf[4..6].copy_from_slice(&id.to_be_bytes());
    buf[6..8].copy_from_slice(&seq.to_be_bytes());

    let plen = payload.len().min(MAX_PING_PAYLOAD);
    buf[ICMP_HEADER_LEN..ICMP_HEADER_LEN + plen].copy_from_slice(&payload[..plen]);

    let total_len = ICMP_HEADER_LEN + plen;
    let sum = internet_checksum(&buf[..total_len]);
    buf[2..4].copy_from_slice(&sum.to_be_bytes());

    buf
}

pub struct IcmpEchoReply {
    pub id: u16,
    pub seq: u16,
    pub payload_len: usize,
}

pub fn parse_echo_reply(data: &[u8]) -> Option<IcmpEchoReply> {
    if data.len() < ICMP_HEADER_LEN {
        return None;
    }
    if data[0] != ICMP_ECHO_REPLY {
        return None;
    }
    let id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    let payload_len = data.len().saturating_sub(ICMP_HEADER_LEN);
    Some(IcmpEchoReply { id, seq, payload_len })
}
