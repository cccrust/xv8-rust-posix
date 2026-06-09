# Dock8 — 容器管理工具

`dock8` 是 xv8 的容器 CLI 工具，靈感來自 Docker，用於管理輕量級隔離環境（容器）。它提供使用者空間的容器生命週期管理命令。

## 命令

| 命令 | 功能 |
|------|------|
| `run <image> <cmd>` | 建立並啟動新容器 |
| `exec <id> <cmd>` | 在運行中的容器執行命令 |
| `ps` | 列出所有容器 |
| `rm <id>` | 移除容器 |
| `images` | 列出可用映像 |

## 底層機制

`dock8` 依賴 xv8 核心提供的隔離原語：

- **namespace**: PID、UTS、mount 等命名空間提供資源隔離
- **cgroup**: 控制組限制 CPU、記憶體使用
- **pivot_root**: 切換容器根檔案系統
- **overlayfs**: 聯合掛載提供分層檔案系統

## 相關文件

- [container.rs](../../testbin/container.md) — 容器測試
- [ns_pid.md](../../testbin/ns_pid.md) — PID 命名空間
- [pivot_root.md](../../testbin/pivot_root.md) — Root 切換
- [overlay.md](../../testbin/overlay.md) — OverlayFS
- [cgroup.md](../../testbin/cgroup.md) — 控制組
