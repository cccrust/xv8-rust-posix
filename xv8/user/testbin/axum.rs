#![no_std]
#![no_main]

use xv8_http::{Response, StatusCode};
use xv8_http::parse::parse_request;
use xv8_router::{handler_fn, Router};
use xv8_tokio_compat::io::{AsyncReadExt, AsyncWriteExt};
use xv8_tokio_compat::runtime::Runtime;
use xv8_tokio_compat::TcpListener;

use user::*;

const PORT: u16 = 27002;

fn check(test: &str, ok: bool) {
    if ok {
        println!("  {} ... ok", test);
    } else {
        println!("  {} ... FAILED", test);
        exit(1);
    }
}

fn serve() {
    let rt = Runtime::new();
    rt.block_on(async {
        let listener = TcpListener::bind(PORT)
            .await
            .expect("bind");

        let app = Router::new()
            .get("/", handler_fn(|| async {
                Response::new(StatusCode::OK)
                    .header("content-type", b"text/plain")
                    .body("ok\n".into())
            }));

        let (mut stream, _) = listener.accept().await.expect("accept");

        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).await.expect("read");
        check("http request received", n > 0);

        let (req, _) = parse_request(&buf[..n]).expect("parse");
        let handler = app.find(&req);
        let resp = handler(req).await;
        let resp_bytes = resp.to_bytes();
        stream.write_all(&resp_bytes).await.expect("write_all");

        println!("  serve: done");
    });
}

fn client() {
    let cli = tcp_socket().expect("client socket");
    tcp_connect(cli, &kernel::abi::Ipv4Addr::LOOPBACK.0, PORT).expect("connect");

    let req = b"GET / HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n";
    let _ = tcp_send(cli, req);

    let mut buf = [0u8; 256];
    let n = tcp_recv(cli, &mut buf).expect("recv");
    let response = core::str::from_utf8(&buf[..n]).unwrap_or("");
    check("response contains ok", response.contains("ok"));

    close(cli).expect("close");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("_axum: async HTTP server (Router API)...");

    match fork().expect("fork") {
        0 => {
            serve();
            exit(0);
        }
        _parent_pid => {
            let _ = nanosleep(0, 200_000_000);
            client();

            let mut status = 0;
            wait(&mut status).expect("wait");
            if status == 0 {
                println!("_axum: PASS");
                exit(0);
            } else {
                println!("_axum: FAILED (server exit={})", status);
                exit(1);
            }
        }
    }
}
