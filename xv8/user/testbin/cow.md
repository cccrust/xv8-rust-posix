# Cow — 寫入時複製測試

`cow` 測試驗證 xv8 的寫入時複製（Copy-on-Write, COW）機制，這是虛擬記憶體與 `fork` 系統呼叫的效能關鍵。`fork` 建立子行程時不立即複製父行程的實體頁面，而是將兩者的頁表指向同一唯讀頁面。當任一方寫入時觸發 page fault，核心才複製該頁面。COW 大幅減少 `fork` 的開銷。

## 相關文件

- [vm.md](../../kernel/src/vm.md) — 虛擬記憶體管理
- [sbrk.md](./sbrk.md) — 堆積擴展測試
- [proc.md](../../kernel/src/proc.md) — 行程管理
