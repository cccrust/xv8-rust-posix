use crate::io::{self, ErrorKind};
use xv8_libc;

pub struct UdpSocket {
    fd: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddr {
    pub ip: [u8; 4],
    pub port: u16,
}

impl SocketAddr {
    pub const fn new(ip: [u8; 4], port: u16) -> Self {
        Self { ip, port }
    }

    pub fn parse(s: &str) -> Result<Self, ()> {
        let (ip_str, port_str) = s.rsplit_once(':').ok_or(())?;
        let port: u16 = port_str.parse().map_err(|_| ())?;
        let mut ip = [0u8; 4];
        for (i, octet) in ip_str.split('.').enumerate() {
            if i >= 4 {
                return Err(());
            }
            ip[i] = octet.parse().map_err(|_| ())?;
        }
        Ok(Self { ip, port })
    }
}

impl UdpSocket {
    pub fn bind(port: u16) -> io::Result<Self> {
        let ret = xv8_libc::socket(port);
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(Self { fd: ret as usize })
        }
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let ret = xv8_libc::raw::send(
            self.fd,
            buf.as_ptr(),
            buf.len(),
            addr.ip.as_ptr(),
            addr.port,
        );
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(ret as usize)
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut src_ip = [0u8; 4];
        let mut src_port: u16 = 0;
        let ret = xv8_libc::raw::receive(
            self.fd,
            buf.as_mut_ptr(),
            buf.len(),
            src_ip.as_mut_ptr(),
            &mut src_port as *mut u16,
        );
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok((
                ret as usize,
                SocketAddr {
                    ip: src_ip,
                    port: src_port,
                },
            ))
        }
    }

    pub fn fd(&self) -> usize {
        self.fd
    }
}
