use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::io::{self, ErrorKind, Read, Write};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd};
use xv8_libc;

// ─── IP address types ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddrParseError(());

impl fmt::Display for AddrParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid address")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self { octets: [a, b, c, d] }
    }

    pub const UNSPECIFIED: Self = Self { octets: [0, 0, 0, 0] };
    pub const LOOPBACK: Self = Self { octets: [127, 0, 0, 1] };
    pub const BROADCAST: Self = Self { octets: [255, 255, 255, 255] };

    pub fn octets(&self) -> [u8; 4] { self.octets }
    pub fn is_loopback(&self) -> bool { self.octets[0] == 127 }
    pub fn is_private(&self) -> bool {
        self.octets[0] == 10
            || (self.octets[0] == 172 && (16..=31).contains(&self.octets[1]))
            || (self.octets[0] == 192 && self.octets[1] == 168)
    }
    pub fn is_unspecified(&self) -> bool { self.octets == [0, 0, 0, 0] }
    pub fn is_broadcast(&self) -> bool { self.octets == [255, 255, 255, 255] }
    pub fn is_link_local(&self) -> bool { self.octets[0] == 169 && self.octets[1] == 254 }
    pub fn is_multicast(&self) -> bool { self.octets[0] & 0xF0 == 224 }
    pub fn to_ipv6_compatible(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, u16::from_be_bytes([self.octets[0], self.octets[1]]), u16::from_be_bytes([self.octets[2], self.octets[3]]))
    }
    pub fn to_ipv6_mapped(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, u16::from_be_bytes([self.octets[0], self.octets[1]]), u16::from_be_bytes([self.octets[2], self.octets[3]]))
    }
}

impl From<[u8; 4]> for Ipv4Addr {
    fn from(octets: [u8; 4]) -> Self { Self { octets } }
}

impl From<Ipv4Addr> for [u8; 4] {
    fn from(ip: Ipv4Addr) -> Self { ip.octets }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.octets[0], self.octets[1], self.octets[2], self.octets[3])
    }
}

impl FromStr for Ipv4Addr {
    type Err = AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 { return Err(AddrParseError(())); }
        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part.parse().map_err(|_| AddrParseError(()))?;
        }
        Ok(Ipv4Addr { octets })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ipv6Addr {
    segments: [u16; 8],
}

impl Ipv6Addr {
    pub const fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Self { segments: [a, b, c, d, e, f, g, h] }
    }

    pub const UNSPECIFIED: Self = Self { segments: [0; 8] };
    pub const LOOPBACK: Self = Self { segments: [0, 0, 0, 0, 0, 0, 0, 1] };

    pub fn segments(&self) -> [u16; 8] { self.segments }
    pub fn octets(&self) -> [u8; 16] {
        let mut oct = [0u8; 16];
        for (i, &seg) in self.segments.iter().enumerate() {
            oct[i * 2] = (seg >> 8) as u8;
            oct[i * 2 + 1] = seg as u8;
        }
        oct
    }
    pub fn is_loopback(&self) -> bool { self.segments == [0, 0, 0, 0, 0, 0, 0, 1] }
    pub fn is_unspecified(&self) -> bool { self.segments == [0; 8] }
    pub fn is_multicast(&self) -> bool { self.segments[0] & 0xff00 == 0xff00 }
    pub fn is_link_local(&self) -> bool { self.segments[0] & 0xffc0 == 0xfe80 }
    pub fn is_unique_local(&self) -> bool { (self.segments[0] & 0xfe00) == 0xfc00 }
    pub fn is_global(&self) -> bool {
        !self.is_unspecified()
            && !self.is_loopback()
            && !self.is_multicast()
            && !self.is_link_local()
            && !self.is_unique_local()
    }
}

impl From<[u8; 16]> for Ipv6Addr {
    fn from(octets: [u8; 16]) -> Self {
        let mut seg = [0u16; 8];
        for i in 0..8 {
            seg[i] = u16::from_be_bytes([octets[i * 2], octets[i * 2 + 1]]);
        }
        Self { segments: seg }
    }
}

impl From<[u16; 8]> for Ipv6Addr {
    fn from(segments: [u16; 8]) -> Self { Self { segments } }
}

impl fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Find the longest run of zero segments for compact notation
        let mut best_start = 8;
        let mut best_len = 0;
        let mut cur_start = 8;
        let mut cur_len = 0;
        for (i, &seg) in self.segments.iter().enumerate() {
            if seg == 0 {
                if cur_len == 0 { cur_start = i; }
                cur_len += 1;
                if cur_len > best_len { best_start = cur_start; best_len = cur_len; }
            } else {
                cur_len = 0;
            }
        }
        if best_len < 2 { best_start = 8; best_len = 0; }
        for mut i in 0..8 {
            if i == best_start {
                if i == 0 { write!(f, "::")?; } else { write!(f, ":")?; }
                i += best_len - 1;
            } else if i > 0 {
                write!(f, ":")?;
            }
            if i < best_start || i >= best_start + best_len {
                write!(f, "{:x}", self.segments[i])?;
            }
        }
        Ok(())
    }
}

impl FromStr for Ipv6Addr {
    type Err = AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "::" { return Ok(Self::UNSPECIFIED); }
        if s == "::1" { return Ok(Self::LOOPBACK); }
        Err(AddrParseError(()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    pub fn is_loopback(&self) -> bool {
        match self { IpAddr::V4(v4) => v4.is_loopback(), IpAddr::V6(v6) => v6.is_loopback() }
    }
    pub fn is_unspecified(&self) -> bool {
        match self { IpAddr::V4(v4) => v4.is_unspecified(), IpAddr::V6(v6) => v6.is_unspecified() }
    }
    pub fn is_multicast(&self) -> bool {
        match self { IpAddr::V4(v4) => v4.is_multicast(), IpAddr::V6(v6) => v6.is_multicast() }
    }
    pub fn is_global(&self) -> bool {
        match self { IpAddr::V4(v4) => !v4.is_private() && !v4.is_loopback() && !v4.is_unspecified() && !v4.is_link_local(), IpAddr::V6(v6) => v6.is_global() }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(v4: Ipv4Addr) -> Self { IpAddr::V4(v4) }
}

impl From<Ipv6Addr> for IpAddr {
    fn from(v6: Ipv6Addr) -> Self { IpAddr::V6(v6) }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { IpAddr::V4(v4) => v4.fmt(f), IpAddr::V6(v6) => v6.fmt(f) }
    }
}

impl FromStr for IpAddr {
    type Err = AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            s.parse::<Ipv6Addr>().map(IpAddr::V6)
        } else {
            s.parse::<Ipv4Addr>().map(IpAddr::V4)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    pub const fn new(ip: Ipv4Addr, port: u16) -> Self { Self { ip, port } }
    pub fn ip(&self) -> &Ipv4Addr { &self.ip }
    pub fn port(&self) -> u16 { self.port }
    pub fn set_ip(&mut self, ip: Ipv4Addr) { self.ip = ip; }
    pub fn set_port(&mut self, port: u16) { self.port = port; }
}

impl fmt::Display for SocketAddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddrV6 {
    ip: Ipv6Addr,
    port: u16,
    flowinfo: u32,
    scope_id: u32,
}

impl SocketAddrV6 {
    pub const fn new(ip: Ipv6Addr, port: u16, flowinfo: u32, scope_id: u32) -> Self {
        Self { ip, port, flowinfo, scope_id }
    }
    pub fn ip(&self) -> &Ipv6Addr { &self.ip }
    pub fn port(&self) -> u16 { self.port }
    pub fn flowinfo(&self) -> u32 { self.flowinfo }
    pub fn scope_id(&self) -> u32 { self.scope_id }
    pub fn set_ip(&mut self, ip: Ipv6Addr) { self.ip = ip; }
    pub fn set_port(&mut self, port: u16) { self.port = port; }
    pub fn set_flowinfo(&mut self, flowinfo: u32) { self.flowinfo = flowinfo; }
    pub fn set_scope_id(&mut self, scope_id: u32) { self.scope_id = scope_id; }
}

impl fmt::Display for SocketAddrV6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]:{}", self.ip, self.port)
    }
}

// ─── SocketAddr (existing, extended) ─────────────────────────────────────

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

    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::new(self.ip[0], self.ip[1], self.ip[2], self.ip[3])
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn set_ip(&mut self, ip: Ipv4Addr) {
        self.ip = ip.octets();
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn is_ipv4(&self) -> bool { true }
    pub fn is_ipv6(&self) -> bool { false }
}

impl From<(Ipv4Addr, u16)> for SocketAddr {
    fn from((ip, port): (Ipv4Addr, u16)) -> Self {
        Self { ip: ip.octets(), port }
    }
}

impl From<SocketAddrV4> for SocketAddr {
    fn from(v4: SocketAddrV4) -> Self {
        Self { ip: v4.ip().octets(), port: v4.port() }
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

impl ToSocketAddrs for Ipv4Addr {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new(self.octets(), 0)))
    }
}

impl ToSocketAddrs for Ipv6Addr {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new([127, 0, 0, 1], 0)))
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new(self.0.octets(), self.1)))
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = alloc::vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        let ip: Ipv4Addr = self.0.parse().map_err(|_| ErrorKind::InvalidInput)?;
        Ok(alloc::vec![SocketAddr::new(ip.octets(), self.1)].into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new(self.ip().octets(), self.port())))
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
