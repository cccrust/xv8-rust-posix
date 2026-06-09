# Async — 非同步執行測試

`async` 測試驗證 xv8 的非同步執行環境（async runtime）是否正常運作。測試內容包括 `async`/`.await` 語法支援、Future trait 實作、Waker 機制與執行器排程。非同步程式設計依賴協同式多工（cooperative multitasking），任務在 await 點自願讓出控制權。

## 相關文件

- [io_async.md](../../../xv8rust/xv8-async/src/io_async.md) — Async I/O
- [reactor.md](../../../xv8rust/xv8-async/src/reactor.md) — Reactor 事件驅動
