#[cfg(target_arch = "riscv64")]
fn main() {
    // Compile-time verification that xv8-tokio-compat types and traits are accessible.
    // Full runtime testing is done by the _httpepoll and _axum testbins in xv8 QEMU.
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use xv8_tokio_compat::io::{AsyncRead, AsyncWrite, ReadBuf};
    use xv8_tokio_compat::runtime::Runtime;
    use xv8_tokio_compat::sync::oneshot;
    use xv8_tokio_compat::{TcpListener, TcpStream};
    use xv8_user_std::io::Result;

    fn _assert_kinds() {
        fn _assert_send<T: Send>() {}
        _assert_send::<TcpStream>();
        _assert_send::<TcpListener>();
        _assert_send::<Runtime>();

        fn _assert_async_read<T: AsyncRead>() {}
        fn _assert_async_write<T: AsyncWrite>() {}
        _assert_async_read::<TcpStream>();
        _assert_async_write::<TcpStream>();

        fn _assert_oneshot() {
            let (tx, rx) = oneshot::channel::<()>();
            fn _assert_send_sender<T: Send>(_t: T) {}
            fn _assert_send_receiver<T: Send>(_t: T) {}
            _assert_send_sender(tx);
            _assert_send_receiver(rx);
        }
    }

    fn _read_buf_usage() {
        let mut buf = [0u8; 64];
        let mut read_buf = ReadBuf::new(&mut buf);
        let _cap = read_buf.capacity();
        let _init = read_buf.initialized();
        let _remaining = read_buf.remaining_mut();
        read_buf.advance(10);
    }

    fn _trait_methods(stream: Pin<&mut TcpStream>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let mut buf = [0u8; 64];
        let mut read_buf = ReadBuf::new(&mut buf);
        stream.poll_read(cx, &mut read_buf)
    }
}

#[cfg(not(target_arch = "riscv64"))]
use axum::{routing::get, Router};

#[cfg(not(target_arch = "riscv64"))]
use std::{error::Error, time::Duration};

#[cfg(not(target_arch = "riscv64"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(not(target_arch = "riscv64"))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new().route("/", get(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = tokio::net::TcpStream::connect(addr).await?;
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await?;

    let mut response = String::new();
    client.read_to_string(&mut response).await?;
    assert!(response.contains("ok"), "unexpected response: {response}");

    let _ = shutdown_tx.send(());
    server.await?;

    Ok(())
}
