# Netstat — 網路統計工具

`netstat` 顯示網路連線狀態、路由表與網路介面統計資訊。它讀取系統中的 socket 狀態並以表格形式呈現，顯示每個連線的協定（TCP/UDP）、本地位址、外部位址、狀態（LISTEN、ESTABLISHED、TIME_WAIT 等）與 PID。網路位址轉換函式根據 netstat 手冊頁格式輸出。

## 相關文件

- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
- [tcp.md](../../kernel/src/net/tcp.md) — TCP 狀態
