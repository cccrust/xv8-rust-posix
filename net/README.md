# 網路工具集 — net/

本專案提供 xv8 作業系統的網路工具集，包含 15 個網路工具與一個通訊協定函式庫，可在 xv8 的 TCP/UDP 堆疊上執行。

## 背景

xv8 的核心具有完整的網路堆疊：Ethernet + ARP + IPv4 + ICMP + UDP + TCP，搭配 E1000 PCIe 網路卡驅動程式及 QEMU user-mode NAT。net/ 在此基礎上實作常見的網路工具與協定。

## 架構

```
net/
├── Cargo.toml        # 工作空間 (tools + libnet)
├── test.sh           # 煙霧測試 (dns, ntp, tcp echo)
├── libnet/            # 通訊協定函式庫
│   ├── src/
│   │   ├── lib.rs    # 入口
│   │   ├── dns.rs    # DNS 解析
│   │   ├── icmp.rs   # ICMP/Ping
│   │   ├── net_impl.rs
│   │   ├── ntp.rs    # NTP 時間同步
│   │   ├── tftp.rs   # TFTP 檔案傳輸
│   │   ├── util.rs   # 共用工具
│   │   └── proto/    # 協定資料結構
│   │       ├── mod.rs
│   │       ├── dns.rs
│   │       ├── icmp.rs
│   │       └── util.rs
└── tools/            # 網路工具二進位檔案
    ├── Cargo.toml
    └── src/
        ├── ssh_server.rs
        └── bin/      # 15 個工具
```

## 工具列表

| 工具 | 檔案 | 用途 |
|------|------|------|
| `curl` | `bin/curl.rs` | HTTP 客戶端 (支援 GET/POST) |
| `dns` | `bin/dns.rs` | DNS 查詢工具 |
| `host` | `bin/host.rs` | 主機名稱解析 |
| `http_server` | `bin/http_server.rs` | 簡易 HTTP 伺服器 |
| `httpd` | `bin/httpd.rs` | HTTP 守護行程 |
| `httpget` | `bin/httpget.rs` | HTTP GET 請求 |
| `netstat` | `bin/netstat.rs` | 網路連線狀態 |
| `ntp` | `bin/ntp.rs` | NTP 時間同步客戶端 |
| `ping` | `bin/ping.rs` | ICMP Ping |
| `ssh_client` | `bin/ssh_client.rs` | SSH 客戶端 |
| `tcpclient` | `bin/tcpclient.rs` | TCP 客戶端測試 |
| `tcpserver` | `bin/tcpserver.rs` | TCP 伺服器測試 |
| `tftp` | `bin/tftp.rs` | TFTP 客戶端 |
| `traceroute` | `bin/traceroute.rs` | 路由追蹤 |
| `whois` | `bin/whois.rs` | WHOIS 查詢 |
| `ssh_server` | `ssh_server.rs` | SSH 伺服器 (src/ 下) |

## 通訊協定函式庫 (libnet)

`libnet/` 提供不依賴標準函式庫的協定實作：

- **DNS**: 支援 A 記錄查詢、CNAME 解析，使用 UDP 封包直接查詢
- **ICMP**: Ping 封包建構與回應解析
- **NTP**: RFC 5905 時間同步協定，使用 UDP 連接埠 123
- **TFTP**: RFC 1350 簡易檔案傳輸協定

## 交叉編譯

```bash
# RISC-V 目標
cargo build --release --no-default-features --features xv8 \
  -Zbuild-std=core,alloc --target riscv64gc-unknown-none-elf

# Host 目標
cargo build --release
```

`--no-default-features` 會關閉依賴主機 libc 的功能，啟用 `xv8` feature 則使用 `xv8-libc-compat`。

## 測試

```bash
cd net && ./test.sh  # 煙霧測試 (依序測試 dns, ntp, tcp echo)
```

在 QEMU 中完整驗證需執行 `xv8/test_net.sh` (需要 posix + net 工具皆已建置)。

## 相關文件

- [Wiki: 網路堆疊](../_wiki/net/Network-Stack.md)
- [Wiki: TCP](../_wiki/net/TCP.md)
- [Wiki: UDP](../_wiki/net/UDP.md)
- [Wiki: ARP](../_wiki/net/ARP.md)
- [Wiki: Ethernet](../_wiki/net/Ethernet.md)
- [xv8 網路核心](../xv8/kernel/src/net/)
- [計劃版本記錄](_doc/)
