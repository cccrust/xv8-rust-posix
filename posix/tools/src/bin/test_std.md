# Test Std — 標準函式庫測試

`test_std` 驗證 xv8 的 POSIX 工具是否能正確使用標準系統資源與 I/O 操作。測試包括動態記憶體分配、檔案 I/O、環境變數讀寫、時間函式、流程控制。這確保工具在 xv8 核心上的運行行為與標準 Unix 環境一致。

## 相關文件

- [test_dir.md](./test_dir.md) — 目錄測試
- [test_ls.md](./test_ls.md) — ls 測試
- [std.md](../../../xv8rust/xv8-user-std/src/lib.md) — xv8 標準函式庫
