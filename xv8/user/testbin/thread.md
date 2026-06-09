# Thread — 執行緒測試

`thread` 測試驗證 xv8 的執行緒（thread）支援，透過 `clone` 系統呼叫建立共用位址空間的執行緒。執行緒是 CPU 排程的基本單位，共用父行程的虛擬記憶體與檔案描述符但擁有獨立的堆疊與暫存器狀態。測試驗證 POSIX thread 語意（pthread 相容）的實作正確性。

## 相關文件

- [proc.md](../../kernel/src/proc.md) — 行程管理
- [thread_v3.md](./thread_v3.md) — Thread v3 測試
- [sync.md](../../../xv8rust/xv8-user-std/src/sync.md) — 同步原語
