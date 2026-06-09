# Sbrk — 堆積擴展測試

`sbrk` 測試驗證 xv8 的程式堆積（heap）管理。`sbrk`/`brk` 是傳統 Unix 系統中用於調整行程資料段（data segment）大小的系統呼叫，讓 C 語言的 `malloc` 可在執行期擴展堆積。現代 Linux 使用 `mmap` 分配大區塊，但 `sbrk` 仍是程序初始化與小型分配的基礎。xv8 驗證堆積分配的邊界條件與記憶體保護。

## 相關文件

- [vm.md](../../kernel/src/vm.md) — 虛擬記憶體管理
- [cow.md](./cow.md) — COW 測試
- [memlayout.md](../../kernel/src/memlayout.md) — 記憶體佈局
