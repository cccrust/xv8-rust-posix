# Whoami — 目前有效使用者名稱

`whoami`（who am I）輸出目前行程的有效使用者名稱（effective UID）。不同於 `logname`（登入名稱），`whoami` 反應的是經過 `su`、`sudo` 或 `setuid` 變更後的有效身分。若有效 UID 為 0 則輸出 `root`。

## 相關文件

- [who.md](./who.md) — 登入資訊
- [logname.md](./logname.md) — 登入名稱
- [id.md](./id.md) — UID/GID 資訊
