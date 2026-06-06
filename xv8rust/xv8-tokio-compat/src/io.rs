use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use xv8_user_std::io::Result;

#[macro_export]
macro_rules! ready {
    ($e:expr) => {
        match $e {
            Poll::Ready(v) => v,
            Poll::Pending => return Poll::Pending,
        }
    };
}

pub struct ReadBuf<'a> {
    buf: &'a mut [u8],
    filled: usize,
}

impl<'a> ReadBuf<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, filled: 0 }
    }

    pub fn filled(&self) -> &[u8] {
        &self.buf[..self.filled]
    }

    pub fn remaining_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.filled..]
    }

    pub fn advance(&mut self, n: usize) {
        self.filled += n;
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn initialized(&self) -> &[u8] {
        &self.buf[..self.filled]
    }
}

pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>>;
}

pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>>;

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>>;

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>>;
}

pub trait AsyncReadExt: AsyncRead + Unpin {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a, Self>
    where
        Self: Sized,
    {
        ReadFuture { reader: self, buf }
    }
}

impl<T: AsyncRead + Unpin + ?Sized> AsyncReadExt for T {}

pub struct ReadFuture<'a, T: ?Sized> {
    reader: &'a mut T,
    buf: &'a mut [u8],
}

impl<T: AsyncRead + Unpin + ?Sized> Future for ReadFuture<'_, T> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        let this = self.get_mut();
        let mut read_buf = ReadBuf::new(this.buf);
        ready!(Pin::new(&mut *this.reader).poll_read(cx, &mut read_buf))?;
        Poll::Ready(Ok(read_buf.filled().len()))
    }
}

pub fn poll_read<R: AsyncRead + Unpin + ?Sized>(
    reader: &mut R,
    cx: &mut Context<'_>,
    buf: &mut [u8],
) -> Poll<Result<usize>> {
    let mut read_buf = ReadBuf::new(buf);
    ready!(Pin::new(reader).poll_read(cx, &mut read_buf))?;
    Poll::Ready(Ok(read_buf.filled().len()))
}

pub trait AsyncWriteExt: AsyncWrite + Unpin {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> WriteAllFuture<'a, Self>
    where
        Self: Sized,
    {
        WriteAllFuture { writer: self, buf, offset: 0 }
    }
}

impl<T: AsyncWrite + Unpin + ?Sized> AsyncWriteExt for T {}

pub struct WriteAllFuture<'a, T: ?Sized> {
    writer: &'a mut T,
    buf: &'a [u8],
    offset: usize,
}

impl<T: AsyncWrite + Unpin + ?Sized> Future for WriteAllFuture<'_, T> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.get_mut();
        while this.offset < this.buf.len() {
            let n = ready!(Pin::new(&mut *this.writer).poll_write(cx, &this.buf[this.offset..]))?;
            if n == 0 {
                return Poll::Ready(Err(xv8_user_std::io::ErrorKind::WriteZero.into()));
            }
            this.offset += n;
        }
        Poll::Ready(Ok(()))
    }
}

use crate::TcpStream;

impl AsyncRead for TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let n = ready!(self.poll_read_inner(cx, buf.remaining_mut()))?;
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        self.poll_write_inner(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}
