# Eventfd — 事件通知檔案描述符測試

`eventfd` 測試驗證 eventfd 機制，這是 Linux 提供的輕量級事件通知工具。eventfd 建立一個特殊的檔案描述符，使用者可在其上進行 `read`/`write` 以原子方式傳遞計數值。eventfd 常與 `epoll`/`poll` 搭配使用，作為使用者空間非同步事件的信號機制，其核心優點是無需分配 pipe 的緩衝區。

## 相關文件

- [eventfd.md](../../kernel/src/eventfd.md) — 核心 eventfd 實作
- [poll.md](../../kernel/src/poll.md) — I/O 多路複用
