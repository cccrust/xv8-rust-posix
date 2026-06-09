# Listen — 網路監聽展示

`listen` 是一個簡單的網路監聽展示程式，在 xv8 上建立 TCP 或 UDP socket 並監聽特定埠號，展示 socket API 的正確運作。

## Socket API 展示

此工具序列展示：

1. `socket()` — 建立 socket
2. `bind()` — 綁定位址與埠號
3. `listen()` — 設定監聽佇列（TCP）
4. `accept()` — 接受連線（TCP）
5. `recv()`/`send()` — 收發資料

`listen` 工具主要用於驗證核心網路協定棧的系統呼叫層與 socket 管理功能。

## 相關文件

- [tcp_echo.md](./tcp_echo.md) — TCP Echo 伺服器
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
- [tcp.md](../../kernel/src/net/tcp.md) — TCP 協定
