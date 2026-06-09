# Mesg — 訊息接收控制

`mesg` 控制是否允許其他使用者透過 `write` 或 `talk` 傳送訊息到當前終端機。`mesg y` 允許（預設），`mesg n` 拒絕。這是 Unix 多人系統中的基本隱私與不受打擾機制，控制 `write` 系統呼叫的權限檢查。

## 相關文件

- [write_.md](./write_.md) — 訊息傳送
- [mailx.md](./mailx.md) — 郵件工具
