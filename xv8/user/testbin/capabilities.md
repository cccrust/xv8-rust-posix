# Capabilities — 能力機制測試

`capabilities` 測試驗證 xv8 的 Linux 能力（capability）模型。傳統 Unix 使用全有或全無的 root 權限，而能力模型將超級使用者權限分割為獨立單元（如 `CAP_NET_RAW`、`CAP_SYS_ADMIN`）。測試驗證行程能否正確取得、丟棄與檢查特定能力，以及核心是否在能力不足時正確拒絕操作。

## 相關文件

- [capability.md](../../kernel/src/capability.md) — 核心能力實作
- [seccomp.md](./seccomp.md) — Seccomp 安全過濾
