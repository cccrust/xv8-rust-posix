# Who — 登入使用者資訊

`who` 顯示目前登入系統的使用者資訊，包括使用者名稱、終端機、登入時間、遠端主機（若遠端登入）。讀取 `/var/run/utmp` 或 `/var/log/wtmp`。`who -b` 顯示最後一次系統開機時間，`who -r` 顯示目前的 run-level。`who am i` 顯示當前使用者的身份。

## 相關文件

- [users.md](./users.md) — 使用者列表
- [whoami.md](./whoami.md) — 目前使用者名稱
- [last.md](./last.md) — 登入記錄
