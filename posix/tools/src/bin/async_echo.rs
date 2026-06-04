fn main() {
    #[cfg(not(target_arch = "riscv64"))]
    {
        println!("async_echo: only supported on riscv64");
    }

    #[cfg(target_arch = "riscv64")]
    {
        use std::net::SocketAddr;
        use std::println;
        use xv8_async::net::AsyncTcpListener;

        let port = 27001u16;

        let rt = xv8_async::Runtime::new();
        rt.block_on(async move {
            let listener = AsyncTcpListener::bind(SocketAddr::new([127, 0, 0, 1], port))
                .expect("async_echo: bind failed");
            println!("async_echo: listening on port {}", port);

            loop {
                let (stream, _addr) =
                    listener.accept().await.expect("async_echo: accept failed");
                println!("async_echo: accepted connection");

                let mut buf = [0u8; 4096];
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        println!("async_echo: connection closed");
                    }
                    Ok(n) => {
                        let _ = stream.write(&buf[..n]).await;
                    }
                    Err(_e) => {
                        println!("async_echo: read error");
                    }
                }
            }
        });
    }
}
