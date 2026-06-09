# Users — 登入使用者列表

`users` 列出目前登入的使用者名稱，重複登入的使用者會重複出現。相較於 `who` 的詳細輸出，`users` 僅輸出簡潔的登入使用者清單。此命令讀取 `/var/run/utmp` 檔案取得目前登入 session。

## 相關文件

- [who.md](./who.md) — 詳細登入資訊
- [whoami.md](./whoami.md) — 目前有效使用者
- [logname.md](./logname.md) — 登入名稱
