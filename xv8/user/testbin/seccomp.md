# Seccomp — 安全計算模式測試

`seccomp` 測試驗證 xv8 的 seccomp（secure computing mode）BPF 過濾機制。Seccomp 允許行程設定系統呼叫過濾規則，限制行程只能使用特定的系統呼叫。Google Chrome 最早採用此技術沙箱化瀏覽器外掛程式。Seccomp 使用 BPF（Berkeley Packet Filter）程式碼定義過濾策略，核心在每次系統呼叫時檢查是否符合規則。

## 相關文件

- [seccomp.md](../../kernel/src/seccomp.md) — 核心 seccomp 實作
- [capabilities.md](./capabilities.md) — 能力機制測試
