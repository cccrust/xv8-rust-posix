# SSH Server — SSH 伺服器

`ssh_server.rs` 提供 SSH（Secure Shell）伺服器功能的函式庫，支援安全遠端登入與命令執行。

## SSH 協定架構

SSH（RFC 4251–4254）由三層組成：

1. **傳輸層（Transport Layer）**: 金鑰交換（Diffie-Hellman）、伺服器認證（host key）、加密（AES）、完整性（HMAC）
2. **使用者認證層（User Authentication）**: 密碼驗證、公開金鑰認證（RSA/Ed25519）、鍵盤互動
3. **連線層（Connection Layer）**: 通道（channel）管理、shell 請求、port forwarding

## 通道機制

SSH 連線可複用（multiplex）多個通道，每個通道對應一個 shell、命令執行或轉發連線。通道透過全域的連線層管理，使用 channel ID 區分。

## 實作範圍

`ssh_server.rs` 實作 SSH 伺服器端的協定邏輯，包括金鑰交換、使用者認證、通道建立與 shell 互動。網路傳輸部分由上層 `ssh_server` 二進位檔處理。

## 相關文件

- [ssh_client.md](./bin/ssh_client.md) — SSH 客戶端
- [ssh_server.md](./bin/ssh_server.md) — SSH Server 工具
