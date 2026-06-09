# Umask — 檔案權限遮罩設定

`umask`（user file-creation mask）設定或顯示建立新檔案/目錄時的預設權限遮罩。umask 值指定在建立時**移除**的權限位元。例如 `umask 022` 表示移除群組與其他人的寫入權限，因此新檔案為 `644`（rw-r--r--），新目錄為 `755`（rwxr-xr-x）。

## 相關文件

- [chmod.md](./chmod.md) — 變更權限
- [chown.md](./chown.md) — 變更擁有者
