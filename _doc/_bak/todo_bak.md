# xv8-rust-posix 開發路線圖

## 目前狀態（v1.1）

| 子專案 | 狀態 | 說明 |
|--------|------|------|
| xv8 kernel | ✅ v1.1 | RISC-V kernel, 107+ syscalls, UDP/E1000 |
| xv8 native user | ✅ v1.1 | init, sh, 20+ bins, 10 tests |
| posix/tools | ✅ v0.19 | 138 POSIX tools, cross-compiles for xv8 |
| net/tools | ⚠️ v1.0 | macOS host 端工具（ping/dns/ntp/tftp） |
| xv8_std | ✅ v0.6 | std overlay: io, fs, path, env, process, time, net |
| xv8-libc | ✅ v1.0 | 45+ syscall wrappers (no_std) |

---

## 短期（v1.2 — v1.3）：xv8 網路工具補完

### v1.2 ─ ICMP/ping + listen 工具

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **xv8 ping** | `xv8/user/bin/ping.rs` — ICMP echo request/reply (raw socket) | 🔴 高 |
| **xv8 listen** | `xv8/user/bin/listen.rs` — UDP listen (配合 hostfwd) | 🔴 高 |
| **ICMP kernel path** | kernel ICMP echo reply generation (目前 ipv4.rs 丟棄 ICMP) | 🔴 高 |
| **hostfwd test** | host `nc -u 127.0.0.1 19999` → xv8 listen 驗證 | 🟡 中 |
| **xv8-user-std net 補完** | `net.rs` 增加 `SocketAddr::parse()` & `set_read_timeout` | 🟡 中 |

### v1.3 ─ NTP + 網路工具鏈完整化

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **xv8 ntp** | NTP client → 同步系統時間 (kernel `clock_settime`) | 🔴 高 |
| **xv8 netcat** | `nc`-like UDP/TCP tool (僅 UDP 初期) | 🟡 中 |
| **kernel clock_settime 修復** | 確認 syscall 正確寫入 RTC | 🟡 中 |
| **DHCP 非阻塞等待** | 用 `Channel` sleep/wakeup 取代 busy-wait polling | 🟢 低 |
| **net/libnet 雙後端** | DnsResolver trait + StdDnsResolver / Xv8DnsResolver | 🟢 低 |

---

## 中期（v1.4 — v1.6）：網路工具主機原生 + xv8 同步

### v1.4 ─ ping/ntp/whois/tftp 主機工具

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **net/ping** | macOS `ping` tool（ICMP raw socket, `sudo` fallback guide） | 🔴 高 |
| **net/ntp** | macOS NTP client（`ntp pool.ntp.org`） | 🔴 高 |
| **net/whois** | WHOIS query（`whois example.com`） | 🟡 中 |
| **net/tftp** | TFTP client（get/put） | 🟢 低 |
| **net/test.sh** | 自動化測試腳本 | 🟡 中 |

### v1.5 ─ DHCP + 網路自動化

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **DHCP lease renew** | kernel DHCP lease renewal 支援 | 🟡 中 |
| **E1000 link status** | 在 sys/net 中提供 link status API | 🟢 低 |
| **neteth 測試強化** | 驗證 E1000 RX/TX + loopback 完整路徑 | 🟡 中 |
| **ARP cache 管理** | ARP entry timeout/expiry | 🟢 低 |

### v1.6 ─ xv8_std 網路抽象層完整化

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **TcpStream/TcpListener** | xv8_std TCP stub（僅介面，kernel 無 TCP） | 🟡 中 |
| **net::lookup_host** | DNS 查詢輔助函數 (wrap `/dns` or syscall) | 🟡 中 |
| **Ipv4Addr/ SocketAddr 完整** | 完整的 `std::net::Ipv4Addr` / `SocketAddr` 實作 | 🟡 中 |
| **UdpSocket 補完** | `set_read_timeout`, `connect`, `local_addr` | 🟡 中 |

---

## 長期（v2.0+）：TCP / 進階網路應用

### v2.0 ─ Kernel TCP 協定棧

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **TCP 狀態機** | CLOSED / LISTEN / SYN-SENT / ESTABLISHED / CLOSE-WAIT / TIME-WAIT | 🔴 高 |
| **TCP syscall** | `bind()`, `connect()`, `listen()`, `accept()` syscalls | 🔴 高 |
| **TCP send/recv** | stream-oriented 收送（非 datagram） | 🔴 高 |
| **TCP checksum** | 含 pseudo-header 的 TCP checksum | 🔴 高 |
| **TCP retransmit** | 簡易 RTO + 重傳 | 🟡 中 |
| **xv8 netcat TCP** | `nc host port` via TCP | 🟡 中 |
| **xv8 httpget** | 簡易 HTTP GET client | 🟢 低 |

### v2.1 ─ 效能 & 工具補完

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **xv8_std TcpStream** | 基於 kernel TCP 的 `TcpStream` `TcpListener` | 🔴 高 |
| **sendto/recvfrom syscall** | POSIX 風格的 UDP sendto/recvfrom | 🟡 中 |
| **非阻塞 socket** | `O_NONBLOCK` socket 支援 + poll/select | 🟡 中 |
| **posix/tools 網路工具** | 在 xv8 上跑 `curl`, `wget`-like 工具 | 🟢 低 |
| **Kernel networking performance** | 零拷貝路徑、TX/RX ring 優化 | 🟢 低 |

---

## 其他維護項目

### posix/tools 持續修復

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **bash $((expr)) 括號** | `$((1+1))` 輸出 `2) )` 修正 | 🟡 中 |
| **bash $(cmd) 括號** | 同上，補第二個 `)` | 🟡 中 |
| **bash $HOME 展開** | 環境變數載入 globals | 🟡 中 |
| **posix host tests** | 針對所有 138 工具在 macOS 上執行行為測試 | 🟢 低 |

### xv8-libc 補完

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **net syscall 統一** | socket/send/receive 與 kernel ABI 同步 | 🟢 低 |
| **missing syscalls** | `select`, `poll`, `epoll` stub/noop | 🟢 低 |

### xv8 kernel 維護

| 項目 | 範圍 | 優先級 |
|------|------|--------|
| **COW fork 穩定性** | edge case 測試 | 🟡 中 |
| **E1000 RX 中斷** | 目前 polling，改 interrupt-driven | 🟡 中 |
| **VirtIO 驅動重構** | 抽 shared VirtIO transport layer | 🟢 低 |

---

## 版本時間線（預估）

| 版本 | 預計 | 重點 |
|------|------|------|
| **v1.2** | 2026-06 | ICMP/ping + listen tool |
| **v1.3** | 2026-06 | NTP + 網路工具鏈 |
| **v1.4** | 2026-07 | ping/ntp/whois 主機原生工具 |
| **v1.5** | 2026-07 | DHCP 強化 + 網路自動化 |
| **v1.6** | 2026-07 | xv8_std net 抽象層完整化 |
| **v2.0** | 2026-08+ | Kernel TCP 協定棧 |
| **v2.1** | 2026-08+ | 效能 & TcpStream overlay |

---

## 圖示說明

- 🔴 高 = 下一版優先實作
- 🟡 中 = 有時間再做
- 🟢 低 = 可暫緩，不影響核心功能

總體方向：**v1.x 補完 UDP 工具鏈，v2.x 推進 TCP。**
