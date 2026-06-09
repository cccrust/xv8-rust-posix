# Axum — Axum HTTP 框架測試

`axum` 測試驗證 xv8 的 HTTP 路由器（router）與 axum 相容層在 async 環境中的整合運作。測試涵蓋路由匹配、請求處理器分派、回應產生與 middleware 串接。axum 以 tower 服務抽象為基礎，利用 Rust 的型別系統確保請求/回應處理的安全性。

## 相關文件

- [router.md](../../../xv8rust/xv8-router/src/router.md) — Router 框架
- [http.md](./http.md) — HTTP 測試
- [axum.md (async)](./async.md) — Async 測試
