# Httpepoll — Epoll 非同步 HTTP 測試

`httpepoll` 測試驗證 xv8 的 epoll I/O 事件通知機制與 HTTP 客戶端的組合使用。epoll（event poll）是 Linux 的高效能 I/O 事件通知機制，優於傳統的 `select`/`poll`，能夠在大量檔案描述符場景下維持 O(1) 的監控效能。此測試同時驗證 async runtime 與 epoll reactor 的整合。

## 相關文件

- [http.md](./http.md) — HTTP 請求測試
- [poll.md](../../kernel/src/poll.md) — I/O 多路複用
- [reactor.md](../../../xv8rust/xv8-async/src/reactor.md) — Reactor 實作
