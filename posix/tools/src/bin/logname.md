# Logname — 顯示使用者登入名稱

`logname` 顯示使用者的登入名稱（login name），即當初登入系統時的使用者名稱，而非目前有效 UID 的使用者名稱（由 `whoami` 顯示）。兩者在 `su` 或 `sudo` 執行時可能不同。`logname` 透過讀取 `/var/run/utmp` 或 /dev/tty 取得登入資訊。

## 相關文件

- [whoami.md](./whoami.md) — 目前有效 UID 的名稱
- [who.md](./who.md) — 登入使用者
- [id.md](./id.md) — UID/GID 資訊
