# Handler — 請求處理器

`handler.rs` 定義路由器的請求處理器（handler）類型。處理器是接收 HTTP 請求並產生回應的函式。

## Handler Trait

```rust
trait Handler<T, S, B> {
    type Future: Future<Output = Response<Body>>;
    fn call(&self, req: Request<T>, state: S) -> Self::Future;
}
```

- **T**: 請求主體類型
- **S**: 共享狀態類型（如資料庫連線池、設定）
- **B**: 回應主體類型

## 提取器（Extractor）模式

Router 支援提取器模式，讓 handler 可宣告其所需的資料：

```rust
async fn handler(Path(id): Path<u64>, Query(params): Query<Params>) -> impl IntoResponse {
    // 使用 id 與 params
}
```

提取器透過 Rust 的型別推導自動從請求中提取資料（路徑參數、查詢參數、JSON body）。

## 相關文件

- [router.md](./router.md) — 路由器
- [into_response.md](./into_response.md) — 回應轉換
- [lib.md](./lib.md) — Router 總覽
