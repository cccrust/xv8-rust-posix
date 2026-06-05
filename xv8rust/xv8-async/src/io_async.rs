use core::pin::Pin;
use core::task::{Context, Poll};
use xv8_libc;
use xv8_user_std::io::{Error, ErrorKind};
use xv8_user_std::net::SocketAddr;
use xv8_user_std::os::unix::io::AsRawFd;

use crate::reactor;

#[derive(Debug)]
pub struct AsyncTcpStream {
    fd: usize,
    peer: SocketAddr,
}

impl AsyncTcpStream {
    pub async fn connect<A: xv8_user_std::net::ToSocketAddrs>(addr: A) -> Result<Self, Error> {
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
            return Err(ErrorKind::Other.into());
        }

        xv8_libc::fcntl(fd, xv8_libc::F_SETFL, xv8_libc::O_NONBLOCK);

        Ok(Self { fd, peer })
    }

    pub fn from_tcp_stream(stream: xv8_user_std::net::TcpStream) -> Self {
        let fd = stream.as_raw_fd() as usize;
        let peer = stream.peer_addr().unwrap_or(SocketAddr::new([0, 0, 0, 0], 0));
        xv8_libc::fcntl(fd, xv8_libc::F_SETFL, xv8_libc::O_NONBLOCK);
        core::mem::forget(stream);
        Self { fd, peer }
    }

    pub fn poll_read(self: Pin<&mut Self>, cx: &Context, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
        let ret = xv8_libc::tcp_recv(self.fd, buf.as_mut_ptr(), buf.len());
        if ret >= 0 {
            return Poll::Ready(Ok(ret as usize));
        }
        let err_code = (-ret) as u16;
        if err_code == xv8_libc::EAGAIN {
            reactor::register_read(self.fd, cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(Err(ErrorKind::Other.into()))
    }

    pub fn poll_write(self: Pin<&mut Self>, cx: &Context, buf: &[u8]) -> Poll<Result<usize, Error>> {
        let ret = xv8_libc::tcp_send(self.fd, buf.as_ptr(), buf.len());
        if ret >= 0 {
            return Poll::Ready(Ok(ret as usize));
        }
        let err_code = (-ret) as u16;
        if err_code == xv8_libc::EAGAIN {
            reactor::register_write(self.fd, cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(Err(ErrorKind::Other.into()))
    }

    pub fn fd(&self) -> usize {
        self.fd
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.peer)
    }
}

impl Drop for AsyncTcpStream {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}

#[derive(Debug)]
pub struct AsyncTcpListener {
    fd: usize,
    addr: SocketAddr,
}

impl AsyncTcpListener {
    pub async fn bind<A: xv8_user_std::net::ToSocketAddrs>(addr: A) -> Result<Self, Error> {
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

        xv8_libc::fcntl(fd, xv8_libc::F_SETFL, xv8_libc::O_NONBLOCK);

        Ok(Self { fd, addr })
    }

    pub fn poll_accept(self: Pin<&Self>, cx: &Context) -> Poll<Result<(AsyncTcpStream, SocketAddr), Error>> {
        let ret = xv8_libc::tcp_accept(self.fd);
        if ret >= 0 {
            let child_fd = ret as usize;
            xv8_libc::fcntl(child_fd, xv8_libc::F_SETFL, xv8_libc::O_NONBLOCK);
            let stream = AsyncTcpStream {
                fd: child_fd,
                peer: SocketAddr::new([0, 0, 0, 0], 0),
            };
            return Poll::Ready(Ok((stream, self.addr)));
        }
        let err_code = (-ret) as u16;
        if err_code == xv8_libc::EAGAIN {
            reactor::register_read(self.fd, cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(Err(ErrorKind::Other.into()))
    }

    pub fn fd(&self) -> usize {
        self.fd
    }

    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        Ok(self.addr)
    }
}

impl Drop for AsyncTcpListener {
    fn drop(&mut self) {
        let _ = xv8_libc::close(self.fd);
    }
}