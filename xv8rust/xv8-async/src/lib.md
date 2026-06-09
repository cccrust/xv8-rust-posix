# Lib — 非同步執行期

`lib.rs` 是 xv8-async crate 的根模組，提供適用於 xv8 的輕量級非同步執行期（async runtime）。

## 非同步 Rust

Rust 的非同步模型是**零成本抽象**——使用 `async fn` 與 `Future` 不會引入額外的堆積分配或執行期開銷。xv8-async 提供讓 Future 真正運作所需的執行期元件：

- **Executor**: 負責輪詢（poll）所有註冊的 Future
- **Reactor**: 監控 I/O 事件並喚醒對應的 Future
- **Waker**: 標準化的喚醒機制

## 設計哲學

xv8-async 遵循最小實作原則，僅提供在 xv8 上運行 async 程式所需的最少元件，不追求與 tokio 或 async-std 的功能對等。

## 相關文件

- [reactor.md](./reactor.md) — I/O 事件驅動核心
- [io_async.md](./io_async.md) — 非同步 I/O 抽象
- [lib.md](../../xv8-tokio-compat/src/lib.md) — Tokio 相容層
