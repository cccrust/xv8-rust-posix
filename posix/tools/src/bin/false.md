# False — 傳回非零退出碼

`false` 什麼都不做，僅傳回非零（失敗）退出碼。在 shell 腳本中，`false` 用於建構無限迴圈（`while false`，不過更常見的是 `while true` 的反向）或確保命令鏈失敗（`cmd1 && false && cmd2`）。它是退出碼合約的極簡範例——每個程式執行的退出碼指示成功（0）或失敗（非0）。

## 相關文件

- [true.md](./true.md) — 傳回零退出碼
- [sh.md](./sh.md) — Shell 條件判斷
