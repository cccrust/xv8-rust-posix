# Mpsc — 多生產者單消費者通道

`mpsc.rs` 實作 `std::sync::mpsc` 通道（channel），提供多生產者單消費者（Multi-Producer Single-Consumer）的行程間通訊機制。

## 通道模型

MPSC 通道由 Rust 標準函式庫引入，在 xv8 上基於 POSIX pipe（或 eventfd + futex）實作：

- **Sender**: 可複製，多個生產者可同時發送
- **Receiver**: 唯一消費者，透過 `recv()` 阻塞等待
- **try_recv()**: 非阻塞版本，無資料時立即回傳 `TryRecvError::Empty`

## xv8 的適應

xv8 的 mpsc 實作可能使用核心提供的 pipe 或 eventfd 作為底層通訊媒介。在無多核心（SMP）支援時，通道實際行為為順序而非並行。

## 相關文件

- [sync.md](./sync.md) — 同步原語
- [pipe.md](../../kernel/src/pipe.md) — 核心 pipe 實作
