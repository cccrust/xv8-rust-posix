# Renice — 變更行程優先權

`renice`（reset nice）變更已執行中的行程排程優先權（nice 值）。與 `nice`（僅在啟動時設定的差異）。`renice` 可針對 PID、PGID（行程群組）或 UID（使用者所有行程）調整。僅超級使用者可提高優先權（設定更低 nice 值）。

## 相關文件

- [nice.md](./nice.md) — 以低優先權啟動
- [kill.md](./kill.md) — 行程控制
- [ps.md](./ps.md) — 行程列表
