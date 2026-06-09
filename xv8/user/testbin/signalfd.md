# Signalfd — 訊號檔案描述符測試

`signalfd` 測試驗證 signalfd 機制，此機制將傳統的 Unix signal 非同步處理轉為同步的檔案 I/O 模型。行程建立 signalfd 並指定要監聽的訊號集合後，可透過 `read` 從 fd 中讀取 `signalfd_siginfo` 結構取得訊號資訊，而非註冊傳統的 signal handler。這讓訊號處理可整合進 `poll`/`epoll` 的事件驅動架構。

## 相關文件

- [signalfd.md](../../kernel/src/signalfd.md) — 核心 signalfd 實作
- [eventfd.md](./eventfd.md) — Eventfd 測試
- [signal.md](../../kernel/src/signal.md) — 訊號處理
