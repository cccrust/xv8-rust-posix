# SSH Client — SSH 客戶端

`ssh_client` 實作 SSH 協定的客戶端功能，支援安全遠端登入與命令執行。SSH（RFC 4251–4254）透過公鑰密碼學提供加密通訊通道。此客戶端支援金鑰交換（Diffie-Hellman）、伺服器主機金鑰驗證、使用者認證（密碼或公鑰）與通道建立。

## 相關文件

- [ssh_server.md](../../tools/src/ssh_server.md) — SSH 伺服器函式庫
- [tcpserver.md](./tcpserver.md) — TCP Server
- [ssh_server.md](./ssh_server.md) — SSH 伺服器工具
