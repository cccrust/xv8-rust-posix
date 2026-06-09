# Pidfd — PID 檔案描述符測試

`pidfd` 測試驗證 pidfd（process file descriptor）機制。pidfd 是 Linux 5.3 引入的功能，提供一個不重複的檔案描述符來代表一個行程，取代傳統的 PID 因為 PID 可能被重用。透過 pidfd 可對行程發送訊號（`pidfd_send_signal`）並等待其終止（`pidfd_open` + `poll`），避免 PID 重用競爭條件。

## 相關文件

- [pidfd.md](../../kernel/src/pidfd.md) — 核心 pidfd 實作
- [signalfd.md](./signalfd.md) — Signal fd 測試
- [fd.md](./fd.md) — 檔案描述符測試
