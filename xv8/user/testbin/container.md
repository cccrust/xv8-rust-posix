# Container — 容器功能整合測試

`container` 測試驗證 xv8 容器相關功能的整合運作，包括命名空間（namespace）、控制組（cgroup）、pivot_root 與 overlayfs 的協同操作。容器利用 Linux 核心的隔離原語創造輕量級的虛擬化環境，與傳統 VM 不同，容器共用宿主核心但擁有獨立的 PID、網路、掛載等命名空間。

## 相關文件

- [dock8.md](../bin/dock8.md) — 容器 CLI 工具
- [cgroup.md](./cgroup.md) — 控制組測試
- [pivot_root.md](./pivot_root.md) — Root 切換測試
- [overlay.md](./overlay.md) — OverlayFS 測試
