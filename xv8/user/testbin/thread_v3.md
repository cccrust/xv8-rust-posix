# Thread V3 — 進階執行緒測試

`thread_v3` 測試驗證進階執行緒操作，包括多執行緒並發訪問共享資源、mutex 鎖的正確性、條件變數（condition variable）的等待/通知機制。測試驗證 xv8 的 futex（fast userspace mutex）實作，這是 Linux 中 pthread 同步的核心基礎。Futex 允許執行緒在使用者空間快速鎖定，僅在競爭時陷入核心。

## 相關文件

- [thread.md](./thread.md) — 基礎執行緒測試
- [sync.md](../../kernel/src/sync.md) — 核心同步原語
- [proc.md](../../kernel/src/proc.md) — 行程管理
