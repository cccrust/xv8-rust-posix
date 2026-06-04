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

## v2.2 — xv8rust/net bridge + 網路工具移植 ✅

**已於 `_doc/v2.2.md` 完整記錄。**

### 步驟 1：xv8rust 網路層 ✅
- `xv8rust/xv8-net/` — 最小 `std::net` 相容層
  - `TcpStream`（connect、read、write、set_read_timeout、shutdown、try_clone）
  - `TcpListener`（bind、accept、incoming、local_addr）
  - `UdpSocket`（bind、send_to、recv_from、set_read_timeout）
  - `Ipv4Addr` / `IpAddr` / `SocketAddr` / `SocketAddrV4`
  - `ToSocketAddrs`（支援 &str、String、(IpAddr,u16)、(&str,u16)、u16 等）
  - `lookup_host`（直接 IP 解析；DNS 整合待後續）
  - 外部依賴：`xv8-user-std`（內部委派至其 net/io/types），`xv8-libc`
- 僅提供 `net/` 工具所需的 API，不完整實作整個 `std::net`
- **檔案**：`xv8rust/xv8-net/Cargo.toml`, `src/lib.rs`, `src/net.rs`（492 行）
- **編譯**：host + riscv64gc-unknown-none-elf 皆通過，零警告

### 步驟 2：net/ 工具 riscv64 編譯 ✅
- `net/libnet` 新增 `cfg` 切換：host 用 `std::net`，xv8 用 `xv8rust::net` shim
- `net/tools` 新增 riscv64 targets 與 feature gate

### 步驟 3：QEMU 整合測試 ✅
- 在 QEMU 內執行 `dns`, `tcpclient`, `tcpserver`, `ntp` 等工具
- `_nettools` testbin：fork+exec 驗證 tcpserver+tcpclient 通訊
- root `test.sh` 全專案合併測試（9/9 passed）
- QEMU 12/12 tests passed（含 `_nettools`）

## 版本對照

| 版本 | 主要內容 | 依賴 |
|------|---------|------|
| v2.1 | xv8 核心 TCP + TCP echo 測試 ✅ | v2.0 (核心穩定性) |
| v2.2 | xv8rust/net bridge + net/ 工具移植 ✅ | v2.1 (核心 TCP) |
| v2.3 | HTTP/TLS 支援、curl/http_server 在 xv8 執行 ⬜ | v2.2 |

## v2.4 — xv8rust async runtime + axum smoke test

### 步驟 1：xv8rust async runtime / reactor 骨架 ✅
- `xv8rust/xv8-async/` 或等價模組：建立單執行緒 async runtime 入口
- 提供 `block_on`、`spawn`、`JoinHandle`、`sleep` 與最小 timer/IO reactor
- 直接重用 `xv8rust/xv8-user-std` 已有的 `fd`、`sync`、`time`、`net` 基礎層
- 目標不是完整複製 Tokio，而是先做出可讓 async crate 掛載的最小執行環境
- 已完成：新增 `xv8rust/xv8-async` crate，含單執行緒 executor、task queue、timer reactor、`sleep` / `yield_now` / `spawn` / `block_on`

### 步驟 2：axum 最小 smoke test ✅
- 新增一個最小 HTTP server 測試 crate，先驗證 `tokio` / `axum` 的 API 依賴面
- 以單一 route、單一 listener、單一 request/response 做最小驗證
- 若 upstream crate 有額外需求，先在 smoke test 中列出缺口，再回推 runtime / std overlay
- 已完成：新增 `xv8rust/xv8-axum-smoke`，host 端可用本地 `tokio` / `axum` source 起 server、打 localhost request 並正常結束
- xv8 target 版 smoke harness 仍待後續把同一條路徑接到 `xv8-async`

### 驗收
- `xv8rust` 可在 riscv64 target 下編譯 async runtime scaffold
- 最小 axum smoke test 可在 host 上運行；xv8/QEMU 版是下一步
- 文件明確區分「可驗證的最小鏈路」與「完整 Tokio 相容性」
