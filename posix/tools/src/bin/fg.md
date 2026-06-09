# Fg — 前景工作控制

`fg` 命令將背景或暫停的工作（job）移回前景執行。Unix 的工作控制（job control）起源於 4.2 BSD。每個工作屬於一個行程群組，前景工作群組擁有終端機的控制權。輸入 `fg %1` 將編號 1 的工作移回前景。Shell 的 `wait` 內建命令等待前景工作完成。

## 相關文件

- [bg.md](./bg.md) — 背景執行
- [jobs.md](./jobs.md) — 工作列表
- [sh.md](./sh.md) — Shell
