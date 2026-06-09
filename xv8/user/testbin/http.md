# Http — HTTP 請求測試

`http` 測試驗證 xv8 的 HTTP/1.1 客戶端功能，測試建立 TCP 連線、發送 HTTP 請求、解析 HTTP 回應的完整流程。HTTP（Hypertext Transfer Protocol）是網際網路上最廣泛使用的應用層協定，定義了客戶端與伺服器之間請求和回應的格式與互動規則，遵循 RFC 9112/9113。

## 相關文件

- [httpepoll.md](./httpepoll.md) — Epoll 模式 HTTP 測試
- [http.md](../../../xv8rust/xv8-http/src/lib.md) — HTTP 類型庫
- [tcp.md](../../kernel/src/net/tcp.md) — TCP 協定
