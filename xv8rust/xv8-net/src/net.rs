use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use xv8_user_std::io::{self, ErrorKind};

// Re-export xv8_user_std I/O traits for convenience
pub use xv8_user_std::io::{Read, Write, Error, ErrorKind as IoErrorKind, Result as IoResult};
pub use xv8_user_std::time::Duration;

// ===== Ipv4Addr =====

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    pub const UNSPECIFIED: Ipv4Addr = Ipv4Addr { octets: [0, 0, 0, 0] };
    pub const LOCALHOST: Ipv4Addr = Ipv4Addr { octets: [127, 0, 0, 1] };
    pub const BROADCAST: Ipv4Addr = Ipv4Addr { octets: [255, 255, 255, 255] };

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        Ipv4Addr { octets: [a, b, c, d] }
    }

    pub const fn octets(&self) -> [u8; 4] {
        self.octets
    }
}

impl From<[u8; 4]> for Ipv4Addr {
    fn from(octets: [u8; 4]) -> Self {
        Ipv4Addr { octets }
    }
}

impl From<Ipv4Addr> for u32 {
    fn from(addr: Ipv4Addr) -> u32 {
        u32::from_be_bytes(addr.octets)
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
}

impl FromStr for Ipv4Addr {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return Err(());
        }
        let mut octets = [0u8; 4];
        for (i, p) in parts.iter().enumerate() {
            let val: u16 = p.parse().map_err(|_| ())?;
            if val > 255 {
                return Err(());
            }
            octets[i] = val as u8;
        }
        Ok(Ipv4Addr { octets })
    }
}

// ===== IpAddr =====

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpAddr {
    V4(Ipv4Addr),
}

impl IpAddr {
    pub fn is_unspecified(&self) -> bool {
        match self {
            IpAddr::V4(addr) => *addr == Ipv4Addr::UNSPECIFIED,
        }
    }

    pub fn is_loopback(&self) -> bool {
        match self {
            IpAddr::V4(addr) => *addr == Ipv4Addr::LOCALHOST,
        }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(addr: Ipv4Addr) -> IpAddr {
        IpAddr::V4(addr)
    }
}

impl From<[u8; 4]> for IpAddr {
    fn from(octets: [u8; 4]) -> IpAddr {
        IpAddr::V4(Ipv4Addr { octets })
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddr::V4(addr) => addr.fmt(f),
        }
    }
}

impl FromStr for IpAddr {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv4Addr::from_str(s).map(IpAddr::V4)
    }
}

// ===== SocketAddrV4 =====

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    pub fn new(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4 { ip, port }
    }

    pub fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for SocketAddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

// ===== SocketAddr =====

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SocketAddr {
    V4(SocketAddrV4),
}

impl SocketAddr {
    pub fn new(ip: IpAddr, port: u16) -> SocketAddr {
        match ip {
            IpAddr::V4(ip4) => SocketAddr::V4(SocketAddrV4::new(ip4, port)),
        }
    }

    pub fn ip(&self) -> IpAddr {
        match self {
            SocketAddr::V4(addr) => IpAddr::V4(*addr.ip()),
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            SocketAddr::V4(addr) => addr.port(),
        }
    }
}

impl From<SocketAddrV4> for SocketAddr {
    fn from(addr: SocketAddrV4) -> SocketAddr {
        SocketAddr::V4(addr)
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketAddr::V4(addr) => addr.fmt(f),
        }
    }
}

impl FromStr for SocketAddr {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_str, port_str) = s.rsplit_once(':').ok_or(())?;
        let port: u16 = port_str.parse().map_err(|_| ())?;
        let ip: Ipv4Addr = ip_str.parse().map_err(|_| ())?;
        Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
    }
}

// ===== ToSocketAddrs =====

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

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::new(self.0, self.1)))
    }
}

impl ToSocketAddrs for (&str, u16) {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        let ip: Ipv4Addr = self.0.parse().map_err(|_| ErrorKind::InvalidInput)?;
        Ok(core::iter::once(SocketAddr::V4(SocketAddrV4::new(ip, self.1))))
    }
}

impl ToSocketAddrs for (String, u16) {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        (self.0.as_str(), self.1).to_socket_addrs()
    }
}

impl ToSocketAddrs for &str {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        let addr: SocketAddr = self.parse().map_err(|_| ErrorKind::InvalidInput)?;
        Ok(core::iter::once(addr))
    }
}

impl ToSocketAddrs for String {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.as_str().to_socket_addrs()
    }
}

impl ToSocketAddrs for &String {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.as_str().to_socket_addrs()
    }
}

impl ToSocketAddrs for u16 {
    type Iter = core::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(core::iter::once(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            *self,
        ))))
    }
}

// ===== Shutdown =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shutdown {
    Read,
    Write,
    Both,
}

// ===== UdpSocket =====

#[derive(Debug)]
pub struct UdpSocket {
    inner: xv8_user_std::net::UdpSocket,
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        let saddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;
        let port = saddr.port();
        let xv8_addr = xv8_user_std::net::SocketAddr::new([0, 0, 0, 0], port);
        let inner = xv8_user_std::net::UdpSocket::bind(xv8_addr)?;
        Ok(UdpSocket { inner })
    }

    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        let target = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;
        let (ip, port) = match target {
            SocketAddr::V4(inner) => (inner.ip().octets(), inner.port()),
        };
        let xv8_target = xv8_user_std::net::SocketAddr::new(ip, port);
        self.inner.send_to(buf, &xv8_target)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let (n, xv8_addr) = self.inner.recv_from(buf)?;
        let addr = SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from(xv8_addr.ip),
            xv8_addr.port,
        ));
        Ok((n, addr))
    }

    pub fn set_read_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn set_write_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn fd(&self) -> usize {
        self.inner.fd()
    }
}

// ===== TcpStream =====

#[derive(Debug)]
pub struct TcpStream {
    inner: xv8_user_std::net::TcpStream,
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        let saddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;
        let (ip_bytes, port) = match saddr {
            SocketAddr::V4(v4) => (v4.ip().octets(), v4.port()),
        };
        let xv8_addr = xv8_user_std::net::SocketAddr::new(ip_bytes, port);
        let inner = xv8_user_std::net::TcpStream::connect(xv8_addr)?;
        Ok(TcpStream { inner })
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        let addr = self.inner.peer_addr()?;
        Ok(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from(addr.ip),
            addr.port,
        )))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn set_read_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn set_write_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn set_nodelay(&self, yes: bool) -> io::Result<()> {
        self.inner.set_nodelay(yes)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.inner.nodelay()
    }

    pub fn set_nonblocking(&self, yes: bool) -> io::Result<()> {
        self.inner.set_nonblocking(yes)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let xv8_how = match how {
            Shutdown::Read => xv8_user_std::net::Shutdown::Read,
            Shutdown::Write => xv8_user_std::net::Shutdown::Write,
            Shutdown::Both => xv8_user_std::net::Shutdown::Both,
        };
        self.inner.shutdown(xv8_how)
    }

    pub fn try_clone(&self) -> io::Result<TcpStream> {
        let inner = self.inner.try_clone()?;
        Ok(TcpStream { inner })
    }
}

impl io::Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl io::Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

// ===== TcpListener =====

#[derive(Debug)]
pub struct TcpListener {
    inner: xv8_user_std::net::TcpListener,
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let saddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(ErrorKind::InvalidInput)?;
        let port = saddr.port();
        let xv8_addr = xv8_user_std::net::SocketAddr::new([0, 0, 0, 0], port);
        let inner = xv8_user_std::net::TcpListener::bind(xv8_addr)?;
        Ok(TcpListener { inner })
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (inner, _xv8_addr) = self.inner.accept()?;
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
        Ok((TcpStream { inner }, addr))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        let xv8_addr = self.inner.local_addr()?;
        Ok(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::from(xv8_addr.ip),
            xv8_addr.port,
        )))
    }

    pub fn set_nonblocking(&self, yes: bool) -> io::Result<()> {
        self.inner.set_nonblocking(yes)
    }

    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|(stream, _)| stream))
    }
}

// ===== lookup_host =====

/// Resolves a hostname to IP addresses.
/// On xv8, only direct IP parsing is supported at this layer.
pub fn lookup_host(host: &str) -> io::Result<Vec<Ipv4Addr>> {
    if let Ok(ip) = Ipv4Addr::from_str(host) {
        return Ok(vec![ip]);
    }
    Err(io::Error::new(
        ErrorKind::Unsupported,
        "DNS lookup not available in xv8-net; use the xv8 DNS test binary directly",
    ))
}
