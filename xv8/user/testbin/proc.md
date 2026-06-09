# Proc — 行程管理測試

`proc` 測試驗證 xv8 核心的行程管理功能，包括行程建立（`fork`）、行程執行（`exec`）、行程等待（`wait`）、行程終止（`exit`）與行程狀態查詢。行程是作業系統資源分配的基本單位，核心的行程控制區塊（PCB）儲存行程的暫存器狀態、位址空間、開啟檔案列表、訊號處理設定等中繼資料。

## 相關文件

- [proc.md](../../kernel/src/proc.md) — 行程管理核心實作
- [thread.md](./thread.md) — 執行緒測試
- [sched.md](../../kernel/src/sched.md) — 排程器
