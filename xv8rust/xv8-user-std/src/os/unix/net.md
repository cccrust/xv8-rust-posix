# Unix 網路操作

提供 xv8 上的 Unix 領域通訊端（Unix domain socket）支援。
包含 `UnixStream`（串流式通訊端）、`UnixListener`（監聽端）、
`UnixDatagram`（資料報通訊端）等型別。Unix domain socket 使用
檔案系統路徑（如 `/tmp/socket`）作為通訊端點，在同一主機的行程間
提供高效能通訊。此模組對應 Rust 標準程式庫的 `std::os::unix::net` API，
底層透過 xv8-libc 包裝 `socket`、`bind`、`connect`、`accept`、`sendto`/`recvfrom`
等系統呼叫。
