#[cfg(target_arch = "riscv64")]
fn main() {
    use std::net::SocketAddr;
    use std::println;
    use xv8_async::net::AsyncTcpListener;

    let port = 27003u16;

    let rt = xv8_async::Runtime::new();
    rt.block_on(async move {
        let listener = AsyncTcpListener::bind(SocketAddr::new([127, 0, 0, 1], port))
            .expect("axum_smoke: bind failed");
        println!("axum_smoke: listening on port {}", port);

        loop {
            let (stream, _addr) = listener.accept().await.expect("axum_smoke: accept failed");
            println!("axum_smoke: accepted connection");

            let mut buf = [0u8; 4096];
            match stream.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    if buf.starts_with(b"GET") {
                        let _ = stream
                            .write(
                                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nok",
                            )
                            .await;
                    } else {
                        let _ = stream
                            .write(
                                b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                            )
                            .await;
                    }
                }
                _ => {
                    println!("axum_smoke: connection error");
                }
            }
        }
    });
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
