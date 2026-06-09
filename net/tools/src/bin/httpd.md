# Httpd — HTTP 常駐程式

`httpd`（HTTP Daemon）是一個功能較完整的 HTTP 伺服器，支援靜態檔案服務、目錄索引、MIME 類型偵測與基本並行連線處理。不同於 `http_server`，`httpd` 更接近生產級 HTTP 伺服器的最小模型，包括請求路由、錯誤頁面與日誌輸出。

## 相關文件

- [http_server.md](./http_server.md) — HTTP 伺服器
- [httpget.md](./httpget.md) — HTTP GET 工具
- [response.md](../../../xv8rust/xv8-http/src/response.md) — HTTP 回應結構
