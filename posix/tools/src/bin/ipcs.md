# Ipcs — IPC 狀態查詢

`ipcs` 顯示 System V IPC 物件的目前狀態，包括訊息佇列、信號量陣列與共享記憶體區段。輸出顯示每個物件的 key、ID、擁有者、權限與大小。此工具對偵錯 IPC 資源洩漏與確認通訊端點存在性非常重要。

## 相關文件

- [ipcrm.md](./ipcrm.md) — 移除 IPC 物件
- [ipc.md](../../kernel/src/ipc.md) — IPC 核心實作
