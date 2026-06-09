# Cgroup — 控制組測試

`cgroup` 測試驗證 xv8 的控制組（cgroup）機制，確認核心能正確限制行程群的資源使用量（CPU 時間、記憶體上限、I/O 頻寬）。cgroup 是 Linux 容器技術（Docker、LXC）的底層基礎設施，由 Google 工程師 Paul Menage 與 Rohit Seth 於 2006 年提出，後於 2014 年合併入 Linux 核心。

## 相關文件

- [cgroup.md](../../kernel/src/cgroup.md) — 核心 cgroup 實作
- [container.md](./container.md) — 容器整合測試
