use crate::net_impl::{Ipv4Addr, Duration};
use crate::util::internet_checksum;
use std::time::Instant;

pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP6_ECHO_REQUEST: u8 = 128;
pub const ICMP6_ECHO_REPLY: u8 = 129;

#[derive(Debug)]
pub struct IcmpEcho {
    pub typ: u8,
    pub code: u8,
    pub checksum: u16,
    pub id: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl IcmpEcho {
    pub fn new_request(id: u16, seq: u16, payload: &[u8]) -> Self {
        let mut pkt = IcmpEcho {
            typ: ICMP_ECHO_REQUEST,
            code: 0,
            checksum: 0,
            id,
            seq,
            payload: payload.to_vec(),
        };
        pkt.checksum = pkt.compute_checksum();
        pkt
    }

    pub fn new_reply(id: u16, seq: u16, payload: &[u8]) -> Self {
        let mut pkt = IcmpEcho {
            typ: ICMP_ECHO_REPLY,
            code: 0,
            checksum: 0,
            id,
            seq,
            payload: payload.to_vec(),
        };
        pkt.checksum = pkt.compute_checksum();
        pkt
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.payload.len());
        buf.push(self.typ);
        buf.push(self.code);
        buf.extend_from_slice(&self.checksum.to_be_bytes());
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.seq.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        Some(IcmpEcho {
            typ: data[0],
            code: data[1],
            checksum: u16::from_be_bytes([data[2], data[3]]),
            id: u16::from_be_bytes([data[4], data[5]]),
            seq: u16::from_be_bytes([data[6], data[7]]),
            payload: data[8..].to_vec(),
        })
    }

    fn compute_checksum(&self) -> u16 {
        internet_checksum(&self.to_bytes())
    }
}

/// macOS ICMP ping via `SOCK_DGRAM` + `IPPROTO_ICMP` (requires root).
#[cfg(target_os = "macos")]
pub fn ping(
    target: Ipv4Addr,
    id: u16,
    seq: u16,
    timeout: Duration,
) -> Result<(Duration, usize), String> {
    use std::mem;

    let fd = unsafe {
        let fd = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_ICMP);
        if fd < 0 {
            return Err(format!("socket: {}", std::io::Error::last_os_error()));
        }
        let tv = libc::timeval {
            tv_sec: timeout.as_secs() as _,
            tv_usec: timeout.subsec_micros() as _,
        };
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDTIMEO,
            &tv as *const _ as *const libc::c_void,
            mem::size_of::<libc::timeval>() as _,
        );
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &tv as *const _ as *const libc::c_void,
            mem::size_of::<libc::timeval>() as _,
        );
        fd
    };

    let payload = b"abcdefghijklmnopqrstuvwabcdefghi";
    let pkt = IcmpEcho::new_request(id, seq, payload);
    let bytes = pkt.to_bytes();

    let addr = libc::sockaddr_in {
        sin_len: mem::size_of::<libc::sockaddr_in>() as u8,
        sin_family: libc::AF_INET as u8,
        sin_port: 0,
        sin_addr: libc::in_addr {
            s_addr: u32::from(target).to_be(),
        },
        sin_zero: [0i8; 8],
    };

    let start = Instant::now();
    let ret = unsafe {
        libc::sendto(
            fd,
            bytes.as_ptr() as *const libc::c_void,
            bytes.len(),
            0,
            &addr as *const _ as *const libc::sockaddr,
            mem::size_of::<libc::sockaddr_in>() as _,
        )
    };
    if ret < 0 {
        unsafe { libc::close(fd); }
        return Err(format!("sendto: {}", std::io::Error::last_os_error()));
    }

    let mut buf = [0u8; 1024];
    let mut src_addr = unsafe { mem::zeroed::<libc::sockaddr_in>() };
    let mut addr_len = mem::size_of::<libc::sockaddr_in>() as _;
    let ret = unsafe {
        libc::recvfrom(
            fd,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            0,
            &mut src_addr as *mut _ as *mut libc::sockaddr,
            &mut addr_len,
        )
    };
    unsafe { libc::close(fd); }
    if ret < 0 {
        return Err(format!("recvfrom: {}", std::io::Error::last_os_error()));
    }
    let elapsed = start.elapsed();

    let reply = IcmpEcho::from_bytes(&buf[..ret as usize]).ok_or("parse failed")?;
    if reply.typ != ICMP_ECHO_REPLY {
        return Err(format!("unexpected type {}", reply.typ));
    }

    Ok((elapsed, reply.payload.len()))
}
