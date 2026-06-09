# Pipe — 管線通訊測試

`pipe` 測試驗證 xv8 的 Unix pipe（管線）實作。Pipe 是 Unix 系統最古老的 IPC（行程間通訊）機制之一，由 Doug McIlroy 於 1972 年提出。Pipe 提供單向的位元組串流通訊通道——寫入寫入端，從讀取端讀出。Shell 中 `cmd1 | cmd2` 的語法即建立在 pipe 機制之上。

## 相關文件

- [pipe.md](../../kernel/src/pipe.md) — 核心 pipe 實作
- [primes.md](../bin/primes.md) — 管線質數計算
- [sh.md](../../../posix/tools/src/bin/sh.md) — Shell pipe 支援
