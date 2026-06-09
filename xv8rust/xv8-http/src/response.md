# Response — HTTP 回應

`response.rs` 定義 HTTP 回應（Response）的資料結構，包含狀態行（version、status、reason）、標頭與主體。

## 回應結構

```rust
struct Response<T> {
    status: StatusCode,
    version: Version,
    headers: HeaderMap,
    body: T,
}
```

## 狀態碼類別

HTTP 狀態碼依首位數字分為五類：

| 類別 | 範圍 | 意義 |
|------|------|------|
| 1xx | 100–199 | 資訊性（Information） |
| 2xx | 200–299 | 成功（Success） |
| 3xx | 300–399 | 重定向（Redirection） |
| 4xx | 400–499 | 用戶端錯誤（Client Error） |
| 5xx | 500–599 | 伺服器錯誤（Server Error） |

## 回應建構

```rust
Response::builder()
    .status(200)
    .header("Content-Type", "text/plain")
    .body(Body::from("Hello, World!"))
```

## 相關文件

- [status.md](./status.md) — 狀態碼定義
- [header.md](./header.md) — 回應標頭
- [body.md](./body.md) — 回應主體
- [parse.md](./parse.md) — 回應解析
