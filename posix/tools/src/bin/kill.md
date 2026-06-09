# Kill — 終止行程

`kill` 向行程傳送訊號。儘管名稱是「殺死」，但可傳送任意訊號（不一定是 SIGKILL）。預設傳送 SIGTERM（15），請求行程優雅地終止。`kill -9`（SIGKILL）強制終止。`kill -l` 列出所有訊號名稱。訊號是 Unix 行程間非同步通知的基本機制。

## 相關文件

- [signal.md](../../kernel/src/signal.md) — 訊號處理
- [wait.md](./wait.md) — 等待行程
- [ps.md](./ps.md) — 行程列表
