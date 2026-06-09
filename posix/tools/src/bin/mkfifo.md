# Mkfifo — 建立命名管線

`mkfifo`（make FIFO）建立一個 FIFO（First-In First-Out）特殊檔案，也稱為命名管線（named pipe）。與 `pipe` 系統呼叫建立的匿名管線不同，FIFO 存在於檔案系統中，可被不相關的行程打開與通訊。FIFO 在一個行程寫入、另一個行程讀取的典型 producer-consumer 場景中使用。

## 相關文件

- [pipe.md](../../kernel/src/pipe.md) — 核心 pipe 實作
- [sh.md](./sh.md) — Shell 管道 `|`
