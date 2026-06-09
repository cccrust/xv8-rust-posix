# Fd — 檔案描述符測試

`fd` 測試驗證 xv8 核心的檔案描述符管理，包括 fd 的分配、複製（dup）、遞移（passing via UNIX socket）與關閉。檔案描述符是 Unix 行程 I/O 的核心抽象，為非負整數索引指向核心的開啟檔案表格。每個行程有獨立的 fd table，但多個 fd 可指向同一開啟檔案描述（如 `dup` 後）。

## 相關文件

- [fdtable.md](../../kernel/src/fdtable.md) — 檔案描述符表
- [close_range.md](./close_range.md) — 批次關閉測試
- [file.md](../../kernel/src/file.md) — 檔案結構
