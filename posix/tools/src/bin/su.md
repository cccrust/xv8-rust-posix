# Su — 切換使用者

`su`（substitute user）以其他使用者的身份執行 shell。`su` 需要目標使用者的密碼（除非執行者為 root）。`su - username` 模擬完整登入（載入目標使用者的環境）。`su` 使用 `setuid` 系統呼叫變更有效 UID。

## 相關文件

- [newgrp.md](./newgrp.md) — 切換群組
- [login.md](./login.md) — 系統登入
- [id.md](./id.md) — UID/GID 資訊
