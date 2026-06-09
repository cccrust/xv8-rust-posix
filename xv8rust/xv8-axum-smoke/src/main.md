# xv8-axum-smoke 整合測試

此檔案是 xv8 非同步執行緒（xv8-async）與 axum-like 路由器（xv8-router）的編譯期與執行期驗證。
在 RISC-V 目標上僅進行編譯期型別檢查——驗證 `TcpStream`、`TcpListener`、`Runtime`、
`AsyncRead`/`AsyncWrite` trait 以及 `oneshot` channel 的 `Send` 約束是否滿足。
在主機目標上則實際啟動 axum HTTP 伺服器，發送 GET 請求並驗證回應包含 `"ok"`，
確保 xv8-tokio-compat 層與真實 tokio/axum 的 API 相容。
