# Hash — Hash 快取管理

`hash` 命令管理 shell 的內部命令路徑快取。當 shell 執行外部命令時，透過 PATH 搜尋可執行檔並快取路徑以加速後續搜尋。`hash -r` 清除快取（當 PATH 變更時使用），`hash -l` 列出目前快取的內容。

## 相關文件

- [command.md](./command.md) — 命令執行
- [type_.md](./type_.md) — 命令類型查詢
- [sh.md](./sh.md) — Shell
