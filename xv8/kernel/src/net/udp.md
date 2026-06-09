# UDP — 使用者資料報協定（RFC 768）

## 概述

UDP 提供**無連線、不可靠**的資料報傳送服務，標頭僅 8 bytes。無握手、無連線狀態，適合延遲敏感的應用（DNS、NTP、VoIP）。

## UDP 資料報格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| 來源埠 | 16 bits | 發送端埠（可為 0） |
| 目標埠 | 16 bits | 接收端埠 |
| Length | 16 bits | 總長度（含標頭） |
| Checksum | 16 bits | 含偽標頭的檢查碼 |

## Socket 表

UDP socket 表將（IP, port）配對映射到 socket。由於 UDP 無連線，每個封包根據目標埠號分派。

## UDP vs TCP

| 特性 | UDP | TCP |
|------|-----|-----|
| 連線 | 無 | 面向連線 |
| 可靠性 | 不可靠 | 可靠 |
| 標頭 | 8 bytes | 20 bytes |
| 用途 | DNS、串流、NTP | Web、FTP、SSH |

## 相關文件

- [ipv4.md](./ipv4.md) — IP 封裝
- [tcp.md](./tcp.md) — TCP 對比
- [dhcp.md](./dhcp.md) — DHCP（基於 UDP）
