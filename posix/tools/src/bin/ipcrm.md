# Ipcrm — 移除 System V IPC 物件

`ipcrm` 移除 System V IPC（行程間通訊）物件，包括訊息佇列（msg）、信號量（sem）與共享記憶體（shm）。System V IPC 由 AT&T 在 System V 中引入，提供比 POSIX IPC 更基礎的行程間通訊機制。資源洩漏的 IPC 物件需透過 `ipcrm` 或重新開機清理。

## 相關文件

- [ipcs.md](./ipcs.md) — IPC 狀態查詢
- [ipc.md](../../kernel/src/ipc.md) — IPC 核心實作
