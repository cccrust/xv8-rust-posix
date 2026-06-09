# Timerfd — 計時器檔案描述符測試

`timerfd` 測試驗證 timerfd 機制，該機制將計時器到期事件轉為檔案描述符的可讀事件。Timerfd 建立一個可被 `read` 的 fd，到期時返回到期次數。透過 `poll`/`epoll`/`select` 監控 timerfd，應用可將計時器整合進事件驅動迴圈，無需傳統的 `alarm` 訊號或自訂超時計算。

## 相關文件

- [timerfd.md](../../kernel/src/timerfd.md) — 核心 timerfd 實作
- [poll.md](../../kernel/src/poll.md) — I/O 多路複用
- [eventfd.md](./eventfd.md) — Eventfd 測試
