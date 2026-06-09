# Write — 訊息傳送

`write_`（因 Rust 關鍵字衝突更名）將訊息傳送給另一個登入使用者的終端機。`write user [tty]` 進入逐行模式，每行輸入結束時傳送。接收端必須已設定 `mesg y` 允許接收。此命令是早期 Unix 的即時通訊先驅。

## 相關文件

- [mesg.md](./mesg.md) — 訊息接收控制
- [mailx.md](./mailx.md) — 郵件工具
- [talk.md](./talk.md) — 雙向對話工具
