# Io — 輸入輸出抽象

`io.rs` 實作 `std::io` 模組的核心抽象：`Read`、`Write`、`Seek`、`BufRead` trait，以及 `stdin`、`stdout`、`stderr` 的全域實體。

## I/O 抽象模型

Rust 的 `std::io` 模組基於 trait 設計，讓同一組操作可作用於不同類型的 I/O 來源。xv8 的 io 模組將這些 trait 實作在：

- **Stdin/Stdout/Stderr**: 封裝 fd 0/1/2，委派給 `read`/`write` 系統呼叫
- **BufReader/BufWriter**: 在核心系統呼叫之上提供緩衝層，減少 ecall 次數
- **Cursor**: 記憶體中的 seekable 資料流

## xv8 的適應

xv8 的 io 模組不支援非阻塞 I/O（`set_nonblocking`）或非同步 I/O 操作，這些功能由獨立的 xv8-async crate 提供。

## 相關文件

- [fs.md](./fs.md) — 檔案系統
- [net.md](./net.md) — 網路 I/O
- [io_async.md](../../xv8-async/src/io_async.md) — 非同步 I/O
