# Setns — 命名空間加入測試

`setns` 測試驗證 `setns` 系統呼叫，允許一個行程加入已存在的命名空間。此機制是容器管理工具（如 dock8）的關鍵功能：當使用者執行 `exec` 進入運行中的容器時，工具透過 `setns` 將新行程加入容器的 PID、網路、mount 等命名空間。測試驗證跨命名空間的行程檢視與隔離。

## 相關文件

- [ns_pid.md](./ns_pid.md) — PID 命名空間
- [ns_uts.md](./ns_uts.md) — UTS 命名空間
- [namespace.md](../../kernel/src/namespace.md) — 命名空間核心實作
