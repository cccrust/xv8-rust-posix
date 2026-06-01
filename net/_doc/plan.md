# net — 網路工具集

## 目標

實作一組可在 **macOS（主機開發，標準 Rust `std`）** 上編譯並執行的網路工具。
xv8 QEMU 移植為 v1.0 以後的目標。現行 0.x 版本以主機原生運作為主。

## 開發原則

1. **主機優先** — 所有工具先用 `std::net`（UdpSocket, TcpStream）在 macOS 上實作與測試
2. **純 Rust std** — 不依賴外部 C 函式庫，僅用 `std::net` 和 `std::*`
3. **零 xv8 依賴** — 0.x 版本完全不理 xv8 kernel 或 xv8-libc
4. **xv8 移植延後** — v1.0 時再透過抽象層（trait）加入 xv8 syscall 後端

## 版本規劃

| 版本 | 範圍 | 說明 |
|------|------|------|
| v0.1 | 基礎架構 + ping/host/dns | libnet 共用庫 + 3 支 UDP 工具 |
| v0.2 | TCP 工具 | tcpclient, tcpserver |
| v0.3 | 進階工具 | tftp, ntp, whois |
| v1.0 | xv8 移植 | ✅ E1000 + QEMU user-mode UDP 通訊 OK, _neteth 自動測試 |

## 網路工具範圍

非 POSIX，但實用的網路工具：

| 工具 | 用途 | 協定 | 實作 |
|------|------|------|------|
| `ping` | ICMP echo | ICMP (raw socket) | v0.1 |
| `dns` | DNS 查詢（A/AAAA） | DNS over UDP | v0.1 |
| `host` | DNS lookup（簡化版） | DNS over UDP | v0.1 |
| `tcpclient` | TCP 客戶端（連線收送） | TCP | v0.2 |
| `tcpserver` | TCP 伺服器（echo/daytime） | TCP | v0.2 |
| `tftp` | TFTP 客戶端 | TFTP over UDP | v0.3 |
| `ntp` | NTP 客戶端（時間同步） | NTP over UDP | v0.3 |
| `whois` | WHOIS 查詢 | WHOIS over TCP | v0.3 |

## 架構

```
net/
├── _doc/
│   └── plan.md              # 本文件
├── Cargo.toml               # workspace root
├── libnet/                  # 共用網路函式庫
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── dns.rs           # DNS 訊息封裝/解析
│       ├── icmp.rs          # ICMP echo (ping)
│       └── util.rs          # IP 位址格式化、checksum 等工具
├── tools/                   # 二進制工具（workspace members）
│   ├── Cargo.toml
│   └── src/
│       └── bin/
│           ├── ping.rs
│           ├── dns.rs
│           ├── host.rs
│           ├── tcpclient.rs
│           ├── tcpserver.rs
│           └── (後續新增)
└── test.sh                  # 測試腳本
```

## 實作路徑

### v0.1：基礎架構 + ping/host/dns

**Cargo workspace 設定：**
- `net/Cargo.toml`：workspace, members = ["libnet", "tools"]
- `net/libnet/Cargo.toml`：lib crate
- `net/tools/Cargo.toml`：bin crate（依賴 libnet）

**libnet 模組：**
- `dns.rs`：DNS query/response 封包構造與解析（A record, AAAA record）
- `icmp.rs`：ICMPv4 echo request/reply（需要 `SOCK_RAW` / `IPPROTO_ICMP`）
- `util.rs`：IP 字串 ↔ 數字轉換、ICMP checksum、DNS name encoding

**tools：**
- `ping`：ICMP echo → 目標 IP，顯示 RTT 與統計
- `dns`：指定 DNS server + domain → 回傳 IP
- `host`：簡化版 `dns`（自動用系統 DNS server）

**macOS 注意事項：**
- `ping`（ICMP raw socket）需要 `CAP_NET_RAW` 或 `sudo`
- macOS 上 raw socket 需要 `IP_HDR_INCL` 特殊處理
- macOS 上 ICMP socket 可透過 `SOCK_DGRAM` + `IPPROTO_ICMP`（系統處理 IP header）
- 若 raw socket 權限不足，v0.1 ping 可用 UDP 模式或需 `sudo`

### v0.2：TCP 工具

- `tcpclient`：`std::net::TcpStream` 連線、收送、顯示
- `tcpserver`：`std::net::TcpListener` 監聽、echo/daytime/time 服務

### v0.3：進階 UDP/TCP 工具

- `tftp`：TFTP RRQ/WRQ over UDP
- `ntp`：NTP 客戶端（RFC 5905）
- `whois`：WHOIS 查詢（RFC 3912）

### v1.0：xv8 移植

- 加入 xv8-libc syscall wrapper 作為後端
- libnet trait 化，支援 std / xv8 兩種實作
- QEMU virtio-net 驗證

## 測試策略

| 層級 | 方法 |
|------|------|
| 單元測試 | `cargo test`（libnet 模組） |
| 功能測試 | 工具直接執行，比對標準工具（系統 ping, host, nslookup）輸出 |
| 整合測試 | `test.sh` 自動化驗證基本行為 |

**macOS 開發流程：**
1. `cargo build` 檢查編譯
2. 執行工具並比對系統工具行為
3. `cargo test` 跑單元測試

## 非目標

- 完整 TCP/IP 協定棧（僅用 `std::net`）
- TLS/SSL 支援
- HTTP 客戶端/伺服器
- POSIX 相容性（這些工具非 POSIX 規範）
