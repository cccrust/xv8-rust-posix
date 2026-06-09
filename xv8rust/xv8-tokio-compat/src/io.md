# Io — Tokio I/O 相容層

`io.rs` 實作 tokio 的 `AsyncRead` 與 `AsyncWrite` trait，讓 xv8 的 `TcpStream` 與其他 I/O 類型可與 tokio-based 程式碼互通。

## Tokio I/O Trait

Tokio 定義了 `tokio::io::AsyncRead` 與 `tokio::io::AsyncWrite` trait，這些是 tokio 生態系（如 hyper、tonic）的非同步 I/O 基礎。xv8-tokio-compat 為 xv8 的網路與檔案類型實作這些 trait。

## 橋接策略

xv8-tokio-compat 的 io 模組不複製 tokio 的實作，而是將 tokio trait 實作在 xv8 的同步 I/O 類型之上，透過 xv8-async reactor 進行非同步轉換。

## 相關文件

- [lib.md](./lib.md) — Tokio 相容層總覽
- [io_async.md](../../xv8-async/src/io_async.md) — xv8 非同步 I/O
- [reactor.md](../../xv8-async/src/reactor.md) — Reactor 事件驅動
