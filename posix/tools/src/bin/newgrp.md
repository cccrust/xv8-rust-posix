# Newgrp — 登入新群組

`newgrp`（new group）切換目前登入 session 的所屬群組。執行 `newgrp` 會建立一個新的 shell 行程，其真實 GID 變更為指定的群組（需要群組密碼或使用者為該群組成員）。此命令在協同工作環境中切換不同群組權限時使用。

## 相關文件

- [su.md](./su.md) — 切換使用者
- [id.md](./id.md) — 顯示 UID/GID
- [chgrp.md](./chgrp.md) — 變更群組
