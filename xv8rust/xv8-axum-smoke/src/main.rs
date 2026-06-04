#[cfg(target_arch = "riscv64")]
fn main() {}

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
