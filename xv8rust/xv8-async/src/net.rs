use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use xv8_user_std::io;
use xv8_user_std::io::Read as _;
use xv8_user_std::io::Write as _;
use xv8_user_std::net::{SocketAddr, TcpListener, TcpStream};
use xv8_user_std::sync::Mutex;

pub struct AsyncTcpListener {
    inner: TcpListener,
}

impl AsyncTcpListener {
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        TcpListener::bind(addr).map(|inner| Self { inner })
    }

    pub fn accept(&self) -> Accept<'_> {
        Accept { listener: self }
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }
}

pub struct Accept<'a> {
    listener: &'a AsyncTcpListener,
}

impl Future for Accept<'_> {
    type Output = io::Result<(AsyncTcpStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.inner.accept() {
            Ok((stream, addr)) => Poll::Ready(Ok((AsyncTcpStream::new(stream), addr))),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

pub struct AsyncTcpStream {
    inner: Mutex<TcpStream>,
}

impl AsyncTcpStream {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        TcpStream::connect(addr).map(Self::new)
    }

    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture { stream: self, buf }
    }

    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture { stream: self, buf }
    }
}

impl From<TcpStream> for AsyncTcpStream {
    fn from(stream: TcpStream) -> Self {
        Self::new(stream)
    }
}

impl AsyncTcpStream {
    fn new(stream: TcpStream) -> Self {
        Self { inner: Mutex::new(stream) }
    }
}

pub struct ReadFuture<'a> {
    stream: &'a AsyncTcpStream,
    buf: &'a mut [u8],
}

impl Future for ReadFuture<'_> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        Poll::Ready(
            this
                .stream
                .inner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .read(this.buf),
        )
    }
}

pub struct WriteFuture<'a> {
    stream: &'a AsyncTcpStream,
    buf: &'a [u8],
}

impl Future for WriteFuture<'_> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        Poll::Ready(
            this
                .stream
                .inner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .write(this.buf),
        )
    }
}
