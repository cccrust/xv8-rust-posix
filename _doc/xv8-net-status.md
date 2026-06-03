# xv8 網路狀態 (v2.1 進行中)

## 核心網路堆疊 (xv8/kernel/src/net/)

### 已實作

| 協定/功能 | 檔案 | 狀態 |
|-----------|------|------|
| Ethernet II | `eth.rs` | 完整 — 幀解析/序列化、EtherType 分派 |
| ARP | `arp.rs` | 完整 — 請求/回應、64-entry 快取 |
| IPv4 | `ipv4.rs` | 完整 — 標頭解析、checksum、TTL=64 |
| ICMP | `icmp.rs` | Echo Request/Reply 完整 |
| UDP | `udp.rs` | 完整 — socket 表（16 slots）、佇列深度 8、ephemeral ports |
| DHCP 客戶端 | `dhcp.rs` | 完整 — 狀態機（Discover→Offer→Request→Ack） |
| Loopback | `loopback.rs` | 完整 — 127.0.0.1/8 |
| 路由表 | `route.rs` | 完整 — 最長前綴匹配 + 最小 metric |
| 網路介面 | `interface.rs` | 完整 — NetDevice trait、介面註冊 |
| Ping | `ping.rs` | 完整 — 核心 ICMP echo 子系統（16 slots） |

### 系統呼叫

| 編號 | 名稱 | 說明 |
|------|------|------|
| 24 | `socket(port)` | 開啟 UDP socket |
| 25 | `send(fd, buf, len, dest_ip, dest_port)` | 傳送 UDP 資料報 |
| 26 | `receive(fd, buf, len, src_ip, src_port)` | 接收 UDP 資料報（阻塞） |

### TCP 實作（進行中）

TCP 已部分實作於 `tcp.rs`，但編譯尚未通過（borrow checker 問題）：

| 功能 | 狀態 |
|------|------|
| TCP header 結構 | ✅ |
| TCP state machine 定義 | ✅ |
| TCP socket table | ✅ |
| 被動開啟（listen / accept） | ✅ 邏輯完成，borrow 問題待修 |
| 主動開啟（connect / SYN 發送） | ✅ 邏輯完成 |
| 資料傳送（send / recv） | ✅ 邏輯完成 |
| 連線關閉（FIN） | ✅ |
| 重傳計時器 | ❌ 未實作 |
| 壅塞控制 | ❌ 未實作 |

TCP 相關系統呼叫（新增，編號 108-114）：

| 編號 | 名稱 | 說明 |
|------|------|------|
| 108 | `tcp_socket()` | 建立 TCP socket |
| 109 | `tcp_bind(fd, port)` | 綁定埠號 |
| 110 | `tcp_listen(fd)` | 開始監聽 |
| 111 | `tcp_accept(fd)` | 接受連線 |
| 112 | `tcp_connect(fd, ip, port)` | 連線到遠端 |
| 113 | `tcp_send(fd, buf, len)` | 傳送資料 |
| 114 | `tcp_recv(fd, buf, len)` | 接收資料 |

### 已知限制

- 無 IPv6 支援
- 無 IP 分片支援
- ICMP 僅支援 Echo Request/Reply
- QEMU user-mode NAT 限制（無法從外部連入）
- 核心網路執行緒為單一 `net_thread`
- 所有佇列皆為固定大小陣列

## 主機端網路工具 (net/)

net/ 子專案包含 13 個主機端網路工具，使用 `std::net` 實作：

| 工具 | 說明 |
|------|------|
| ping | ICMP ping |
| host/dns | DNS 查詢 |
| tcpclient/tcpserver | TCP 客戶端/伺服器 |
| ntp | NTP 時間同步 |
| whois | WHOIS 查詢 |
| traceroute | 路由追蹤 |
| netstat | 網路狀態 |
| http_server | HTTP 伺服器 |
| curl | HTTP 客戶端 |
| ssh_client | SSH 客戶端（placeholder） |

## xv8 使用者空間網路工具 (xv8/user/bin/)

| 工具 | 說明 |
|------|------|
| udp | UDP 收發測試 |
| dns | DNS A 紀錄查詢（over UDP） |
| ping | ICMP Echo |
| listen | UDP listener |
| traceroute | UDP traceroute |

## QEMU 網路測試

10 項核心測試中，網路相關 4 項：

| 測試 | 狀態 |
|------|------|
| `_net` (UDP socket open/close/send/recv) | ✅ PASS |
| `_neteth` (E1000 + DHCP + UDP echo) | ✅ PASS |
| `_netdns` (DNS query via QEMU proxy) | ✅ PASS |
| `_netping` (ICMP to QEMU gateway) | ✅ PASS |

## v2.1 待辦事項

1. 修復 TCP borrow checker 問題，讓核心編譯通過
2. 建立簡易 TCP echo server 測試工具（xv8/user/bin/）
3. 建立簡易 HTTP client（over TCP）
4. 驗證 TCP 三向交握與資料傳送
5. 將 net/ 工具逐步移植到 xv8 上執行