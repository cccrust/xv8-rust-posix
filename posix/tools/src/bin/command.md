# Command — 執行命令（跳過別名/函數）

`command` 執行指定的命令，跳過 shell 函數與別名查詢，直接執行外部程式或內建命令。這在 shell 腳本中極其有用：當使用者在 shell 函數中需要呼叫與函數同名的外部命令時，`command` 可避免遞迴呼叫。`command -v` 顯示命令路徑。

## 相關文件

- [sh.md](./sh.md) — Shell
- [type_.md](./type_.md) — 顯示命令類型
- [alias.md](./alias.md) — 命令別名
