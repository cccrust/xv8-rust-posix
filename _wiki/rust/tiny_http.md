# tiny_http

輕量級 HTTP 伺服器，net/ 工具使用。

## 專案使用

```toml
[dependencies]
tiny_http = "0.12"
```

## 基本伺服器

```rust
use tiny_http::{Server, Response};

let server = Server::http("0.0.0.0:80").unwrap();

for request in server.incoming_requests() {
    println!("{} {}", request.method(), request.url());

    let response = Response::from_string("Hello!");
    request.respond(response).unwrap();
}
```

## Request

```rust
for request in server.incoming_requests() {
    // 方法
    let method = request.method();

    // URL
    let url = request.url();

    // 標頭
    for (name, value) in request.headers() {
        println!("{}: {}", name, value);
    }

    // 查詢字串
    let query = request.query_string();
}
```

## Response

```rust
use tiny_http::{Response, Header};

let response = Response::from_string("Hello")
    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap());
```

### 檔案回應

```rust
use std::fs::File;
use std::io::Read;

let mut file = File::open("index.html")?;
let mut contents = Vec::new();
file.read_to_end(&mut contents)?;

let response = Response::from_data(contents)
    .with_header(Header::from_bytes(
        &b"Content-Type"[..],
        &b"text/html"[..]
    ).unwrap());
```

### JSON 回應

```rust
let json = r#"{"status": "ok"}"#;
let response = Response::from_string(json)
    .with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
    );
```

## POST 請求

```rust
use std::io::Read;

for request in server.incoming_requests() {
    if request.method() == &tiny_http::Method::Post {
        let mut body = Vec::new();
        request.as_reader().read_to_end(&mut body)?;
        println!("Body: {:?}", body);
    }
}
```

## 狀態碼

```rust
let response = Response::from_string("Not Found")
    .with_status_code(404);

let response = Response::from_string("Moved")
    .with_status_code(301)
    .with_header(
        Header::from_bytes(&b"Location"[..], &b"/new-url"[..]).unwrap()
    );
```

## 本專案使用

### HTTP 伺服器

```rust
use tiny_http::{Server, Response, Header};

let server = Server::http("0.0.0.0:8080").unwrap();

loop {
    let request = server.recv().unwrap().unwrap();
    let path = request.url();

    let response = match path {
        "/" => Response::from_string("OK"),
        "/ping" => Response::from_string("pong"),
        _ => {
            Response::from_string("Not Found")
                .with_status_code(404)
        }
    };

    request.respond(response).unwrap();
}
```

## 靜態檔案服務

```rust
use std::path::Path;

fn serve_file(path: &str) -> Option<Response<std::io::Empty>> {
    let file_path = Path::new(".").join(path.trim_start_matches('/'));
    if file_path.is_file() {
        let mut file = std::fs::File::open(&file_path).ok()?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).ok()?;

        let ext = file_path.extension()?.to_str()?;
        let mime = match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            _ => "text/plain",
        };

        Some(Response::from_data(contents)
            .with_header(Header::from_bytes(
                &b"Content-Type"[..],
                mime.as_bytes()
            ).unwrap()))
    } else {
        None
    }
}
```

## 與 hyper 的比較

| 特性 | tiny_http | hyper |
|------|-----------|-------|
| API 複雜度 | 簡單 | 複雜 |
| 性能 | 中等 | 極高 |
| 功能 | 基礎 | 完整 |
| 非同步 | 否 | 是 |

## 限制

- 同步 API，blocking
- 功能較基礎
- 不支援 HTTP/2

## 適用場景

tiny_http 適合：
- 簡單的 HTTP 伺服器
- 原型開發
- 教學用途

hyper 適合：
- 生產環境
- 高性能需求
- 需要非同步

## 底層

tiny_http 使用標準庫的 `TcpListener`，在單執行緒中處理連接。

## 相關模組

- `hyper`：功能完整的 HTTP 庫
- `tokio`：非同步 runtime