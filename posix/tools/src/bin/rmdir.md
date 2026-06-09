# Rmdir — 移除空目錄

`rmdir`（remove directory）僅移除空目錄。與 `rm -r`（遞迴刪除非空目錄）不同，`rmdir` 的安全保證——若目錄非空則拒絕操作。`rmdir -p` 同時移除父目錄鏈（如 `rmdir -p a/b/c` 若中間目錄在移除後變空）。POSIX 要求目錄在移除前必須為空（僅包含 `.` 和 `..`）。

## 相關文件

- [rm.md](./rm.md) — 移除檔案
- [mkdir.md](./mkdir.md) — 建立目錄
