# Inotify — 檔案監控測試

`inotify` 測試驗證 inotify 檔案系統事件監控機制。inotify 允許應用程式監控檔案或目錄的變更事件（如修改、刪除、移動、屬性變更），無需輪詢（polling）。應用程式建立 inotify fd、加入監控目標後，透過 `read` 從 fd 讀取事件佇列。此機制廣泛用於檔案管理器、即時同步工具與開發工具。

## 相關文件

- [inotify.md](../../kernel/src/inotify.md) — 核心 inotify 實作
- [poll.md](../../kernel/src/poll.md) — I/O 多路複用
