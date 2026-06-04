// Platform abstraction: host uses std::*, xv8 uses xv8_net::net / xv8_user_std

#[cfg(not(feature = "xv8"))]
mod inner {
    pub use std::io::{Read, Write};
    pub use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
    pub use std::time::{Duration, SystemTime, UNIX_EPOCH};
}

#[cfg(feature = "xv8")]
mod inner {
    pub use xv8_net::net::{IpAddr, Ipv4Addr, Read, SocketAddr, TcpListener, TcpStream, UdpSocket, Write};
    pub use xv8_net::net::Duration;
    pub use xv8_user_std::time::{SystemTime, UNIX_EPOCH};
}

pub use inner::*;
