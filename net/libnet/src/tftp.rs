#[cfg(feature = "xv8")]
use alloc::format;
#[cfg(feature = "xv8")]
use alloc::string::{String, ToString};
#[cfg(feature = "xv8")]
use alloc::vec::Vec;

use crate::net_impl::{UdpSocket, Duration};

const OP_RRQ: u16 = 1;
#[allow(dead_code)]
const OP_WRQ: u16 = 2;
const OP_DATA: u16 = 3;
const OP_ACK: u16 = 4;
const OP_ERROR: u16 = 5;
const BLOCK_SIZE: usize = 512;

#[derive(Debug)]
pub enum TftpError {
    FileNotFound,
    AccessViolation,
    DiskFull,
    IllegalOp,
    UnknownTid,
    FileExists,
    NoSuchUser,
}

impl TftpError {
    pub fn code(&self) -> u16 {
        match self {
            TftpError::FileNotFound => 1,
            TftpError::AccessViolation => 2,
            TftpError::DiskFull => 3,
            TftpError::IllegalOp => 4,
            TftpError::UnknownTid => 5,
            TftpError::FileExists => 6,
            TftpError::NoSuchUser => 7,
        }
    }
    pub fn msg(&self) -> &str {
        match self {
            TftpError::FileNotFound => "File not found",
            TftpError::AccessViolation => "Access violation",
            TftpError::DiskFull => "Disk full or allocation exceeded",
            TftpError::IllegalOp => "Illegal TFTP operation",
            TftpError::UnknownTid => "Unknown transfer ID",
            TftpError::FileExists => "File already exists",
            TftpError::NoSuchUser => "No such user",
        }
    }
    pub fn from_code(c: u16) -> Self {
        match c {
            1 => TftpError::FileNotFound,
            2 => TftpError::AccessViolation,
            3 => TftpError::DiskFull,
            4 => TftpError::IllegalOp,
            5 => TftpError::UnknownTid,
            6 => TftpError::FileExists,
            _ => TftpError::NoSuchUser,
        }
    }
}

fn build_rrq(filename: &str, mode: &str) -> Vec<u8> {
    let mut pkt = Vec::new();
    pkt.extend_from_slice(&OP_RRQ.to_be_bytes());
    pkt.extend_from_slice(filename.as_bytes());
    pkt.push(0);
    pkt.extend_from_slice(mode.as_bytes());
    pkt.push(0);
    pkt
}

fn build_ack(block: u16) -> Vec<u8> {
    let mut pkt = Vec::new();
    pkt.extend_from_slice(&OP_ACK.to_be_bytes());
    pkt.extend_from_slice(&block.to_be_bytes());
    pkt
}

/// Download a file from a TFTP server.
///
/// Returns the raw file contents.
pub fn download(server: &str, filename: &str) -> Result<Vec<u8>, String> {
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {}", e))?;
    sock.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("set timeout: {}", e))?;

    // Send RRQ to port 69
    let rrq = build_rrq(filename, "octet");
    sock.send_to(&rrq, format!("{}:{}", server, 69))
        .map_err(|e| format!("send RRQ: {}", e))?;

    let mut data = Vec::new();
    let mut expected_block = 1u16;

    loop {
        let mut buf = [0u8; 516];
        let (n, _) = sock
            .recv_from(&mut buf)
            .map_err(|e| format!("recv: {}", e))?;

        if n < 4 {
            return Err("packet too short".to_string());
        }

        let op = u16::from_be_bytes([buf[0], buf[1]]);
        match op {
            OP_DATA => {
                let block = u16::from_be_bytes([buf[2], buf[3]]);
                if block != expected_block {
                    return Err(format!(
                        "block mismatch: expected {}, got {}",
                        expected_block, block
                    ));
                }
                let chunk = &buf[4..n];
                data.extend_from_slice(chunk);

                // Send ACK
                let ack = build_ack(block);
                sock.send_to(&ack, format!("{}:{}", server, 69))
                    .map_err(|e| format!("send ACK: {}", e))?;

                // If chunk < 512 bytes, it's the last block
                if chunk.len() < BLOCK_SIZE {
                    break;
                }
                expected_block += 1;
            }
            OP_ERROR => {
                let errcode = u16::from_be_bytes([buf[2], buf[3]]);
                let errmsg = String::from_utf8_lossy(&buf[4..n]);
                return Err(format!("TFTP error {}: {}", errcode, errmsg));
            }
            _ => {
                return Err(format!("unexpected opcode {}", op));
            }
        }
    }

    Ok(data)
}
