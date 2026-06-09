# Env — 環境變數操作

`env.rs` 實作 `std::env` 模組的功能，提供環境變數的讀寫、行程參數（args）存取、當前目錄查詢等操作。

在 xv8 中，這些功能映射到核心系統呼叫：`getcwd`（取得當前目錄）、`chdir`（切換目錄）、`getpid`（取得行程 ID）、`sbrk`（堆積擴展）。

## xv8 的適應

xv8 的環境變數儲存在使用者空間的初始堆疊（由核心在 `exec` 時設定），`env::vars()` 遍歷此區塊。由於 xv8 缺少 `/proc/self/exe` 等功能，部分可執行檔路徑查詢行為有限。

## 相關文件

- [process.md](./process.md) — 行程管理
- [fs.md](./fs.md) — 檔案系統
- [xv8-user-std.md](../xv8-user-std.md) — Crate 總覽
