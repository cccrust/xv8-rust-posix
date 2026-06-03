# hyper

HTTP 客戶端/伺服器，net/ 工具使用。

## 專案使用

```toml
[dependencies]
hyper = { version = "0.14", features = ["full"] }
```

## Server

```rust
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello, World!")))
}

#[tokio::main]
async fn main() {
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle))
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
```

## Request / Response

```rust
use hyper::{Request, Response, Body};
use http_body_util::Full;

let response = Response::builder()
    .status(200)
    .header("Content-Type", "text/plain")
    .body(Full::new(Body::from("Hello")))?;
```

## Body

```rust
use hyper::body::Incoming;
use bytes::Bytes;

let body = Body::from(Bytes::from_static(b"Hello"));

// 讀取
let whole = body::to_bytes(body).await?;
let string = String::from_utf8(whole.to_vec())?;
```

## Client

```rust
use hyper::client::HttpConnector;
use hyper::Client;

let client = Client::builder()
    .build::<_, hyper::Body>(HttpConnector::new());

let resp = client.get("http://example.com".parse().unwrap()).await?;
```

## Router (http crate)

```rust
use hyper::Server;
use http::Request;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

let service = ServiceBuilder::new()
    .layer(TraceLayer::new_for_http())
    .service(handle_request);
```

## 本專案使用

### HTTP 伺服器

```rust
use hyper::{Body, Request, Response};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let response = match path {
        "/" => Response::new(Body::from("OK")),
        "/health" => Response::new(Body::from("Healthy")),
        _ => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    };
    Ok(response)
}
```

## HTTP 版本

```rust
use hyper::Version;

let req = Request::builder()
    .version(Version::HTTP_11)
    .body(Body::empty())?;
```

## Headers

```rust
use hyper::header::{HeaderMap, HeaderValue};

let mut headers = HeaderMap::new();
headers.insert("X-Custom-Header", HeaderValue::from_static("value"));
```

## Status Codes

```rust
use hyper::StatusCode;

let status = StatusCode::NOT_FOUND;  // 404
let status = StatusCode::OK;          // 200
let status = StatusCode::INTERNAL_SERVER_ERROR;  // 500
```

## 與其他 HTTP 庫的比較

| 庫 | 設計 | 性能 |
|----|------|------|
| hyper | async/await | 極高 |
| actix-web | actor | 高 |
| rocket | 宣告式 | 中 |

## 中間層

```rust
use tower::{Service, ServiceBuilder};
use tower::limit::RateLimitLayer;

let service = ServiceBuilder::new()
    .rate_limit(100, Duration::from_secs(1))
    .service(handle_request);
```

## 依賴

hyper 需要額外的 runtime 和 body 處理庫：

```toml
[dependencies]
hyper = { version = "0.14", features = ["full"] }
http-body-util = "0.1"
bytes = "1"
```

## 限制

hyper 0.14 不支援 HTTP/2 伺服器，需要 hyper 1.x。

## 未來升級

hyper 1.0 支援 HTTP/2 和更好的效能，但 API 有較大變化。

## 相關模組

- `tokio`：非同步 runtime
- `http`：HTTP 型別
- `tower`：中間層