#![no_std]

extern crate alloc;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use xv8_async::io_async::{AsyncTcpStream as InnerStream, AsyncTcpListener as InnerListener};
use xv8_user_std::io::{ErrorKind, Result};
use xv8_user_std::net::SocketAddr;

pub mod io;
pub mod runtime;
pub mod sync;
pub mod time;

#[derive(Debug)]
pub struct TcpStream(InnerStream);

impl TcpStream {
    pub async fn connect<A: xv8_user_std::net::ToSocketAddrs>(addr: A) -> Result<Self> {
        InnerStream::connect(addr).await.map(TcpStream)
    }

    pub fn from_async(inner: InnerStream) -> Self {
        TcpStream(inner)
    }

    pub fn into_inner(self) -> InnerStream {
        self.0
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Err(ErrorKind::Unsupported.into())
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize> {
        let ret = xv8_libc::tcp_recv(self.0.fd(), buf.as_mut_ptr(), buf.len());
        if ret >= 0 {
            Ok(ret as usize)
        } else if (-ret) as u16 == xv8_libc::EAGAIN {
            Err(ErrorKind::WouldBlock.into())
        } else {
            Err(ErrorKind::Other.into())
        }
    }

    pub fn try_write(&self, buf: &[u8]) -> Result<usize> {
        let ret = xv8_libc::tcp_send(self.0.fd(), buf.as_ptr(), buf.len());
        if ret >= 0 {
            Ok(ret as usize)
        } else if (-ret) as u16 == xv8_libc::EAGAIN {
            Err(ErrorKind::WouldBlock.into())
        } else {
            Err(ErrorKind::Other.into())
        }
    }

    pub fn poll_read_inner(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }

    pub fn poll_write_inner(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    pub fn fd(&self) -> usize {
        self.0.fd()
    }
}

impl Unpin for TcpStream {}

#[derive(Debug)]
pub struct TcpListener(InnerListener);

impl TcpListener {
    pub async fn bind<A: xv8_user_std::net::ToSocketAddrs>(addr: A) -> Result<Self> {
        InnerListener::bind(addr).await.map(TcpListener)
    }

    pub fn from_async(inner: InnerListener) -> Self {
        TcpListener(inner)
    }

    pub fn into_inner(self) -> InnerListener {
        self.0
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        struct AcceptFuture<'a>(&'a InnerListener);

        impl Future for AcceptFuture<'_> {
            type Output = Result<(InnerStream, SocketAddr)>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(InnerStream, SocketAddr)>> {
                Pin::new(self.0).poll_accept(cx)
            }
        }

        let (stream, addr) = AcceptFuture(&self.0).await?;
        Ok((TcpStream(stream), addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<Result<(TcpStream, SocketAddr)>> {
        Pin::new(&self.0).poll_accept(cx).map(|r| {
            r.map(|(s, a)| (TcpStream(s), a))
        })
    }
}

impl Unpin for TcpListener {}
