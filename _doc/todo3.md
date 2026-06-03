# xv8-rust-posix 開發規劃 v2.1+

## v2.1 — xv8 核心網路強化（TCP）

### 步驟 1：修復 TCP borrow checker 問題
- 目標：讓 `xv8/kernel/src/net/tcp.rs` 編譯通過
- 主要問題：`SpinLock` guard 的存活週期跨越 `table.alloc_port()` 與 `table.find_listener()` 的呼叫
- 解法：將 `entry` 的 mutable borrow 限制在最小範圍，先拷貝需要的欄位再釋放 lock

### 步驟 2：TCP echo server 測試工具
- 新增 `xv8/user/bin/tcp_echo.rs` — TCP echo server
- 監聽指定 port，收到資料後原樣送回
- 驗證 TCP 三向交握與資料傳送

### 步驟 3：TCP syscall 完整啟用
- 確保 syscall 編號 108-114 全部註冊
- `tcp_socket()`, `tcp_bind()`, `tcp_listen()`, `tcp_accept()`, `tcp_connect()`, `tcp_send()`, `tcp_recv()`

### 步驟 4：QEMU TCP 測試
- 新增 `_tcpecho` 測試（TCP echo loopback 驗證）
- 新增 `_tcpconn` 測試（TCP 連線到 QEMU gateway）

## v2.2 — xv8_std/net bridge + 網路工具移植

### 步驟 1：xv8_std 網路層
- `xv8_std/xv8-net/` — 最小 `std::net` 相容層
  - `TcpStream`（connect、read、write）
  - `UdpSocket`（bind、send_to、recv_from）
  - `lookup_host`（DNS 查詢）
- 僅提供 `net/` 工具所需的 API，不完整實作整個 `std::net`

### 步驟 2：net/ 工具 riscv64 編譯
- `net/libnet` 新增 `cfg` 切換：host 用 `std::net`，xv8 用 `xv8_std::net` shim
- `net/tools` 新增 riscv64 targets 與 feature gate

### 步驟 3：QEMU 整合測試
- 在 QEMU 內執行 `dns`, `tcpclient`, `tcpserver`, `ntp` 等工具
- 擴充 xv8 測試框架支援網路工具測試

## 版本對照

| 版本 | 主要內容 | 依賴 |
|------|---------|------|
| v2.1 | xv8 核心 TCP + TCP echo 測試 | v2.0 (核心穩定性) |
| v2.2 | xv8_std/net bridge + net/ 工具移植 | v2.1 (核心 TCP) |
| v2.3 | HTTP/TLS 支援、curl/http_server 在 xv8 執行 | v2.2 |