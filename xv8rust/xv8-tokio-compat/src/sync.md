# Sync — Tokio 同步相容層

`sync.rs` 提供與 `tokio::sync` 相容的同步原語，用於 async 上下文中的執行緒間通訊。

## 實作的類型

- **oneshot**: 單次通道，一個發送者發送一個值，一個接收者接收
- **mpsc**: 多生產者單消費者通道（async 版本）
- **Mutex**: tokio 風格的 async Mutex（在 await 點之間保持鎖）
- **Notify**: 輕量級通知機制，類似條件變數但專為 async 設計

## Tokio 同步 vs std 同步

tokio 的同步原語專為 async 上下文設計：當呼叫 `lock()` 或 `recv()` 時，若資源不可用則 yield（`Poll::Pending`）而非阻塞執行緒。這與 `std::sync` 的阻塞行為不同。

## 相關文件

- [sync.md](../../xv8-user-std/src/sync.md) — xv8 std 同步
- [lib.md](./lib.md) — Tokio 相容層總覽
