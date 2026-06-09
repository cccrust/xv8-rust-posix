# Fs — 檔案系統測試

`fs` 測試驗證 xv8 核心檔案系統的操作，包括檔案的建立、開啟、讀寫、關閉、刪除，以及目錄操作（mkdir、rmdir、readdir）。xv8 的檔案系統基於 xv6 的 log-structured 檔案系統設計，使用 logging layer 確保 crash 安全性。測試驗證 inode 管理、block 分配、目錄 entry 遍歷等核心功能。

## 相關文件

- [fs.md](../../kernel/src/fs.md) — 檔案系統實作
- [file.md](../../kernel/src/file.md) — 檔案結構
- [buf.md](../../kernel/src/buf.md) — Buffer cache
- [log.md](../../kernel/src/log.md) — Logging layer
