# Nice — 調整排程優先權

`nice` 以降低的排程優先權執行命令。核心依照優先權分配 CPU 時間——nice 值越高，優先權越低。POSIX 定義 nice 值範圍為 0–39（預設 20）。僅超級使用者可提高優先權（設定負 nice 值）。`renice` 變更執行中的行程優先權。

## 相關文件

- [renice.md](./renice.md) — 變更執行中行程優先權
- [scheduler.md](../../kernel/src/scheduler.md) — 排程器
