# Ns UTS — UTS 命名空間測試

`ns_uts` 測試驗證 xv8 的 UTS（UNIX Time-sharing System）命名空間隔離功能。UTS 命名空間允許每個容器擁有獨立的 hostname 與 domainname。在容器場景中，每個容器可設定自己的主機名稱，且互不影響。測試驗證 `sethostname`/`gethostname` 系統呼叫在命名空間層級的隔離正確性。

## 相關文件

- [namespace.md](../../kernel/src/namespace.md) — 命名空間核心實作
- [ns_pid.md](./ns_pid.md) — PID 命名空間測試
- [container.md](./container.md) — 容器整合測試
