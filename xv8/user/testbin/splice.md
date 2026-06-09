# Splice — 零拷貝資料傳輸測試

`splice` 測試驗證 `splice` 系統呼叫，該呼叫在兩個檔案描述符之間移動資料，無需在核心與使用者空間之間複製資料。這是 Linux 零拷貝（zero-copy）機制之一，透過 page cache 的直接操作大幅提升資料傳輸效率。`splice` 常用於高效能代理伺服器與資料處理管線。

## 相關文件

- [fd.md](./fd.md) — 檔案描述符測試
- [pipe.md](./pipe.md) — 管線測試
- [file.md](../../kernel/src/file.md) — 檔案結構
