# Request — HTTP 請求

`request.rs` 定義 HTTP 請求（Request）的資料結構，包含請求行（method、uri、version）、標頭與主體。

## 請求結構

```rust
struct Request<T> {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    body: T,
}
```

泛型參數 `T` 允許請求攜帶不同形式的主體——從位元組緩衝區（`Vec<u8>`）到串流類型。

## 請求建構

`Request` 支援 builder 模式建構：

```rust
Request::builder()
    .method(Method::GET)
    .uri("https://example.com/")
    .header("Accept", "application/json")
    .body(Body::empty())
```

## HTTP 版本

- HTTP/0.9: 僅 GET、無標頭、無版本號
- HTTP/1.0: RFC 1945，逐個請求建立新 TCP 連線
- HTTP/1.1: RFC 7230-7235，持久連線（keep-alive）、chunked encoding

## 相關文件

- [method.md](./method.md) — HTTP 方法
- [uri.md](./uri.md) — URI 解析
- [body.md](./body.md) — 請求主體
- [header.md](./header.md) — 請求標頭
- [parse.md](./parse.md) — 請求解析
