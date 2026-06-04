#[cfg(feature = "xv8")]
use alloc::{format, vec};
#[cfg(feature = "xv8")]
use alloc::string::{String, ToString};
#[cfg(feature = "xv8")]
use alloc::vec::Vec;

use crate::net_impl::{UdpSocket, Duration, SystemTime, UNIX_EPOCH};

/// NTP timestamp (seconds since 1900-01-01, 32-bit integer part + 32-bit fractional)
#[derive(Debug, Clone, Copy)]
pub struct NtpTimestamp(pub u32, pub u32);

impl NtpTimestamp {
    pub fn to_seconds_f64(&self) -> f64 {
        self.0 as f64 + self.1 as f64 / (1u64 << 32) as f64
    }

    pub fn to_system_time(&self) -> SystemTime {
        // NTP epoch = 1900-01-01, UNIX epoch = 1970-01-01
        // Difference = 2208988800 seconds
        let ntp_unix_diff = 2208988800u64;
        let unix_secs = self.0 as u64 - ntp_unix_diff;
        let nsecs = ((self.1 as u64) * 1_000_000_000) >> 32;
        UNIX_EPOCH + Duration::new(unix_secs, nsecs as u32)
    }
}

#[derive(Debug)]
pub struct NtpPacket {
    pub li: u8,
    pub vn: u8,
    pub mode: u8,
    pub stratum: u8,
    pub poll: u8,
    pub precision: u8,
    pub root_delay: u32,
    pub root_dispersion: u32,
    pub reference_id: u32,
    pub reference_ts: NtpTimestamp,
    pub origin_ts: NtpTimestamp,
    pub receive_ts: NtpTimestamp,
    pub transmit_ts: NtpTimestamp,
}

fn ntp_timestamp_now() -> NtpTimestamp {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() + 2208988800u64; // convert to NTP epoch
    let frac = ((now.subsec_nanos() as u64) << 32) / 1_000_000_000;
    NtpTimestamp(secs as u32, frac as u32)
}

fn build_request() -> Vec<u8> {
    let mut pkt = vec![0u8; 48];
    // LI=0, VN=4, Mode=3 (client)
    pkt[0] = (0 << 6) | (4 << 3) | 3;
    // Transmit timestamp
    let ts = ntp_timestamp_now();
    pkt[40..44].copy_from_slice(&ts.0.to_be_bytes());
    pkt[44..48].copy_from_slice(&ts.1.to_be_bytes());
    pkt
}

fn parse_packet(data: &[u8]) -> Option<NtpPacket> {
    if data.len() < 48 {
        return None;
    }
    Some(NtpPacket {
        li: (data[0] >> 6) & 3,
        vn: (data[0] >> 3) & 7,
        mode: data[0] & 7,
        stratum: data[1],
        poll: data[2],
        precision: data[3],
        root_delay: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
        root_dispersion: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
        reference_id: u32::from_be_bytes([data[12], data[13], data[14], data[15]]),
        reference_ts: NtpTimestamp(
            u32::from_be_bytes([data[16], data[17], data[18], data[19]]),
            u32::from_be_bytes([data[20], data[21], data[22], data[23]]),
        ),
        origin_ts: NtpTimestamp(
            u32::from_be_bytes([data[24], data[25], data[26], data[27]]),
            u32::from_be_bytes([data[28], data[29], data[30], data[31]]),
        ),
        receive_ts: NtpTimestamp(
            u32::from_be_bytes([data[32], data[33], data[34], data[35]]),
            u32::from_be_bytes([data[36], data[37], data[38], data[39]]),
        ),
        transmit_ts: NtpTimestamp(
            u32::from_be_bytes([data[40], data[41], data[42], data[43]]),
            u32::from_be_bytes([data[44], data[45], data[46], data[47]]),
        ),
    })
}

/// Query an NTP server and return the response packet.
pub fn query(server: &str) -> Result<NtpPacket, String> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {}", e))?;
    sock.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("set timeout: {}", e))?;

    let req = build_request();
    sock.send_to(&req, format!("{}:{}", server, 123))
        .map_err(|e| format!("send: {}", e))?;

    let mut buf = [0u8; 48];
    let (n, _) = sock
        .recv_from(&mut buf)
        .map_err(|e| format!("recv: {}", e))?;

    parse_packet(&buf[..n]).ok_or_else(|| "parse failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_size() {
        let req = build_request();
        assert_eq!(req.len(), 48);
        // mode should be 3 (client)
        assert_eq!(req[0] & 7, 3);
    }

    #[test]
    fn test_parse_response() {
        let mut raw = vec![0u8; 48];
        // LI=0, VN=4, Mode=4 (server)
        raw[0] = (0 << 6) | (4 << 3) | 4;
        raw[1] = 1; // stratum
        raw[40] = 0xE0; // transmit timestamp seconds (high byte)
        let pkt = parse_packet(&raw).unwrap();
        assert_eq!(pkt.mode, 4);
        assert_eq!(pkt.stratum, 1);
    }
}
