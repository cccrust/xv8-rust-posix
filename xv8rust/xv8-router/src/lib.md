# Lib — Router 函式庫

`lib.rs` 是 xv8-router crate 的根模組，提供輕量級的 HTTP 路由器框架，靈感來自 axum。

## 設計哲學

xv8-router 遵循 axum 的設計原則：

1. **型別安全**: 提取器（Extractor）與回應轉換（IntoResponse）利用 Rust 的型別系統確保處理鏈型別安全
2. **Tower 服務抽象**: 中間件與 Handler 使用統一的 Service trait
3. **模組化**: 路由、中間件、提取器為可組合的獨立元件

## `#![no_std]` 相容

xv8-router 標記為 `#![no_std]`，可在 xv8 的核心空間或使用者空間中使用，但依賴 `alloc` crate 提供堆積分配。

## 相關文件

- [router.md](./router.md) — 路由器核心
- [handler.md](./handler.md) — 請求處理器
- [into_response.md](./into_response.md) — 回應轉換
- [axum.md](../../user/testbin/axum.md) — Axum 測試
