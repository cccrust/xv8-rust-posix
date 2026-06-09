# Ns PID — PID 命名空間測試

`ns_pid` 測試驗證 xv8 的 PID 命名空間隔離功能。PID namespace 是 Linux 容器的重要隔離機制，讓容器內行程擁有獨立的 PID 編號空間：容器內的 PID 1 在宿主命名空間可能是 PID 42。命名空間內的行程無法看到其他命名空間的行程，也無法對其發送訊號。此測試驗證 `clone`(CLONE_NEWPID) 與 `setns` 系統呼叫。

## 相關文件

- [namespace.md](../../kernel/src/namespace.md) — 命名空間核心實作
- [setns.md](./setns.md) — 命名空間加入測試
- [container.md](./container.md) — 容器整合測試
