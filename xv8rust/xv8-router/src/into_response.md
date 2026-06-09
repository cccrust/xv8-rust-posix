# IntoResponse — 回應轉換

`into_response.rs` 定義 `IntoResponse` trait，用於將各種類型自動轉換為 HTTP 回應。

## IntoResponse Trait

```rust
pub trait IntoResponse {
    fn into_response(self) -> Response<Body>;
}
```

此 trait 讓 handler 可回傳任意類型（字串、JSON、狀態碼、Html 等），無需手動建構 `Response` 物件。

## 標準實作

- **`&str` / `String`**: 以 `text/plain; charset=utf-8` 回傳
- **`StatusCode`**: 無主體的回應
- **`(StatusCode, impl IntoResponse)`**: 自訂狀態碼 + 主體
- **Json<T>**: 自動序列化為 `application/json`
- **Html<String>**: 以 `text/html` 回傳

## 相關文件

- [handler.md](./handler.md) — 請求處理器
- [router.md](./router.md) — 路由器
- [lib.md](./lib.md) — Router 總覽
