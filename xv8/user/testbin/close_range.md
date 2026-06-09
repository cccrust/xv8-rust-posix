# Close Range — 關閉描述符範圍測試

`close_range` 測試驗證 `close_range` 系統呼叫，此為 Linux 5.9 引入的延伸功能，允許一次關閉指定範圍內的所有檔案描述符。傳統的 `close` 一個一個關閉在大量 fd 場景下效率低落，`close_range` 提供批次關閉以改善效能，尤其在 `fork`/`exec` 前的 fd 清理場景。

## 相關文件

- [fd.md](./fd.md) — 檔案描述符測試
- [sysfile.md](../../kernel/src/sysfile.md) — 檔案系統系統呼叫
