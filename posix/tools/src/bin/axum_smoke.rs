fn main() {
    #[cfg(not(target_arch = "riscv64"))]
    {
        println!("axum_smoke: only supported on riscv64");
    }

    #[cfg(target_arch = "riscv64")]
    {
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
                let (stream, _addr) =
                    listener.accept().await.expect("axum_smoke: accept failed");
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
}
