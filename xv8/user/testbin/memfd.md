# Memfd — 記憶體檔案測試

`memfd` 測試驗證 `memfd_create` 系統呼叫，該呼叫建立一個匿名的、基於記憶體的檔案描述符（類似 tmpfs 但不掛載檔案系統）。memfd 的內容完全在記憶體中，不會持久化到磁碟。它可用於秘密分享、檔案映射通訊（IPC），以及安全地處理敏感資料（可設定 `MFD_CLOEXEC` 確保 exec 後自動關閉）。

## 相關文件

- [memfd.md](../../kernel/src/memfd.md) — 核心 memfd 實作
- [fd.md](./fd.md) — 檔案描述符測試
- [fdtable.md](../../kernel/src/fdtable.md) — 檔案描述符表
