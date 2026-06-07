use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::io::{self, ErrorKind, Read, Write};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd};
use xv8_libc;

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

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port)
    }
}

impl FromStr for SocketAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SocketAddr::parse(s)
    }
}

pub trait ToSocketAddrs {
    type Iter: Iterator<Item = SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = core::iter::Once<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(*self))
    }
}

impl ToSocketAddrs for u16 {
    type Iter = core::iter::Once<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new([0, 0, 0, 0], *self)))
    }
}

impl ToSocketAddrs for &str {
    type Iter = core::iter::Once<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        let addr = SocketAddr::parse(self).map_err(|_| ErrorKind::InvalidInput)?;
        Ok(core::iter::once(addr))
    }
}

impl ToSocketAddrs for String {
    type Iter = core::iter::Once<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.as_str().to_socket_addrs()
    }
}

#[derive(Debug)]
pub struct UdpSocket {
    fd: usize,
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;
        let ret = xv8_libc::socket(addr.port);
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(Self { fd: ret as usize })
        }
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let ret = xv8_libc::send(self.fd, buf.as_ptr(), buf.len(), addr.ip.as_ptr(), addr.port);
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(ret as usize)
        }
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut src_ip = [0u8; 4];
        let mut src_port: u16 = 0;
        let ret = xv8_libc::receive(
            self.fd,
            buf.as_mut_ptr(),
            buf.len(),
            src_ip.as_mut_ptr(),
            &mut src_port as *mut u16,
        );
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok((ret as usize, SocketAddr { ip: src_ip, port: src_port }))
        }
    }

    pub fn fd(&self) -> usize {
        self.fd
    }
}

impl AsRawFd for UdpSocket {
    fn as_raw_fd(&self) -> i32 {
        self.fd as i32
    }
}

impl AsFd for UdpSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}

impl IntoRawFd for UdpSocket {
    fn into_raw_fd(self) -> i32 {
        let fd = self.fd as i32;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for UdpSocket {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        Self { fd: fd as usize }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shutdown {
    Read,
    Write,
    Both,
}

#[derive(Debug)]
pub struct TcpStream {
    fd: usize,
    peer: SocketAddr,
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let peer = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;

        let fd = xv8_libc::tcp_socket();
        if fd < 0 {
            return Err(ErrorKind::Other.into());
        }

        let fd = fd as usize;
        let ret = xv8_libc::tcp_connect(fd, peer.ip.as_ptr(), peer.port);
        if ret < 0 {
            let _ = xv8_libc::close(fd);
            Err(ErrorKind::Other.into())
        } else {
            Ok(Self { fd, peer })
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        let fd = xv8_libc::dup(self.fd);
        if fd < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(Self { fd: fd as usize, peer: self.peer })
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.peer)
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn set_nodelay(&self, _yes: bool) -> io::Result<()> {
        Ok(())
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        Ok(false)
    }

    pub fn set_nonblocking(&self, _yes: bool) -> io::Result<()> {
        Ok(())
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    pub fn shutdown(&self, _how: Shutdown) -> io::Result<()> {
        Ok(())
    }

    pub(crate) fn from_raw_fd(fd: usize, peer: SocketAddr) -> Self {
        Self { fd, peer }
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> i32 {
        self.fd as i32
    }
}

impl AsFd for TcpStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = xv8_libc::tcp_recv(self.fd, buf.as_mut_ptr(), buf.len());
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(ret as usize)
        }
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = xv8_libc::tcp_send(self.fd, buf.as_ptr(), buf.len());
        if ret < 0 {
            Err(ErrorKind::Other.into())
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}

impl IntoRawFd for TcpStream {
    fn into_raw_fd(self) -> i32 {
        let fd = self.fd as i32;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        Self { fd: fd as usize, peer: SocketAddr::new([0, 0, 0, 0], 0) }
    }
}

#[derive(Debug)]
pub struct TcpListener {
    fd: usize,
    addr: SocketAddr,
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;

        let fd = xv8_libc::tcp_socket();
        if fd < 0 {
            return Err(ErrorKind::Other.into());
        }

        let fd = fd as usize;
        if xv8_libc::tcp_bind(fd, addr.port) < 0 {
            let _ = xv8_libc::close(fd);
            return Err(ErrorKind::Other.into());
        }
        if xv8_libc::tcp_listen(fd) < 0 {
            let _ = xv8_libc::close(fd);
            return Err(ErrorKind::Other.into());
        }

        Ok(Self { fd, addr })
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let fd = xv8_libc::tcp_accept(self.fd);
        if fd < 0 {
            return Err(ErrorKind::Other.into());
        }

        let stream = TcpStream::from_raw_fd(fd as usize, SocketAddr::new([0, 0, 0, 0], 0));
        Ok((stream, SocketAddr::new([0, 0, 0, 0], 0)))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.addr)
    }

    pub fn set_nonblocking(&self, _yes: bool) -> io::Result<()> {
        Ok(())
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> i32 {
        self.fd as i32
    }
}

impl AsFd for TcpListener {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}

impl IntoRawFd for TcpListener {
    fn into_raw_fd(self) -> i32 {
        let fd = self.fd as i32;
        core::mem::forget(self);
        fd
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: i32) -> Self {
        Self { fd: fd as usize, addr: SocketAddr::new([0, 0, 0, 0], 0) }
    }
}

fn encode_dns_name(name: &str, buf: &mut [u8]) -> usize {
    let mut pos = 0;
    for label in name.split('.') {
        if label.is_empty() { continue; }
        if pos + 1 + label.len() > buf.len() { break; }
        buf[pos] = label.len() as u8;
        buf[pos + 1..pos + 1 + label.len()].copy_from_slice(label.as_bytes());
        pos += 1 + label.len();
    }
    if pos < buf.len() { buf[pos] = 0; pos += 1; }
    pos
}

fn parse_dns_name(data: &[u8], mut pos: usize) -> (String, usize) {
    let mut name = String::new();
    loop {
        if pos >= data.len() { break; }
        let len = data[pos] as usize;
        if len == 0 { pos += 1; break; }
        if (len & 0xC0) == 0xC0 {
            let offset = (((len & 0x3F) as usize) << 8) | data[pos + 1] as usize;
            let (suffix, _) = parse_dns_name(data, offset);
            if !name.is_empty() { name.push('.'); }
            name.push_str(&suffix);
            pos += 2;
            return (name, pos);
        }
        if pos + 1 + len > data.len() { break; }
        if !name.is_empty() { name.push('.'); }
        name.push_str(core::str::from_utf8(&data[pos + 1..pos + 1 + len]).unwrap_or(""));
        pos += 1 + len;
    }
    (name, pos)
}

pub fn lookup_host(name: &str) -> io::Result<Vec<SocketAddr>> {
    let mut addrs = Vec::new();
    if let Some(ip) = parse_dotted_ip(name) {
        addrs.push(SocketAddr { ip, port: 0 });
        return Ok(addrs);
    }
    let server = [8, 8, 8, 8];
    let port = 53u16;
    let mut query = [0u8; 512];
    query[0..12].copy_from_slice(&[
        0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ]);
    let qpos = encode_dns_name(name, &mut query[12..]);
    let qtype_pos = 12 + qpos;
    if qtype_pos + 4 > query.len() { return Err(ErrorKind::InvalidInput.into()); }
    query[qtype_pos..qtype_pos + 4].copy_from_slice(&[0x00, 0x01, 0x00, 0x01]);
    let qlen = qtype_pos + 4;

    let sock = UdpSocket::bind(SocketAddr::new([0, 0, 0, 0], 0))?;
    let dns_addr = SocketAddr::new(server, port);
    sock.send_to(&query[..qlen], &dns_addr)?;

    let mut resp = [0u8; 512];
    let (n, _from) = sock.recv_from(&mut resp)?;
    if n < 12 { return Err(ErrorKind::InvalidData.into()); }

    let ancount = ((resp[6] as u16) << 8) | resp[7] as u16;
    if ancount == 0 { return Ok(addrs); }

    let mut pos = qtype_pos + 4;
    let mut answers_left = ancount;
    while pos < n && answers_left > 0 {
        if (resp[pos] & 0xC0) == 0xC0 { pos += 2; } else { let (_, new_pos) = parse_dns_name(&resp, pos); pos = new_pos; }
        if pos + 10 > n { break; }
        let rtype = ((resp[pos] as u16) << 8) | resp[pos + 1] as u16;
        let rdlen = ((resp[pos + 8] as u16) << 8) | resp[pos + 9] as u16;
        pos += 10;
        if rtype == 1 && rdlen == 4 && pos + 4 <= n {
            addrs.push(SocketAddr { ip: [resp[pos], resp[pos + 1], resp[pos + 2], resp[pos + 3]], port: 0 });
        }
        pos += rdlen as usize;
        answers_left -= 1;
    }
    Ok(addrs)
}

fn parse_dotted_ip(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 { return None; }
    let mut ip = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        let val: u16 = part.parse().ok()?;
        if val > 255 { return None; }
        ip[i] = val as u8;
    }
    Some(ip)
}
