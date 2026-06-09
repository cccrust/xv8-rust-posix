# IPv4 — 網際網路協定第 4 版（RFC 791）

## 概述

IPv4 提供**盡力而為（best-effort）**的封包傳遞服務，無連線、不可靠，可靠性由上層協定（如 TCP）提供。

## IP 封包格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| Version/IHL | 8 bits | 版本 4 + 標頭長度 |
| DSCP/ECN | 8 bits | 服務類型與壅塞通知 |
| Total Length | 16 bits | 封包總長度 |
| Identification | 16 bits | 分段識別碼 |
| Flags + Fragment Offset | 16 bits | 分段控制 |
| TTL | 8 bits | 存活時間 |
| Protocol | 8 bits | 上層協定（1=ICMP, 6=TCP, 17=UDP） |
| Header Checksum | 16 bits | 標頭錯誤偵測 |
| 來源/目標 IP | 64 bits | 位址 |

## 路由決策

1. 目標是否為本機位址？→ 交付上層協定
2. 查詢路由表取得下一跳閘道器
3. 若目標在 LAN，ARP 解析 MAC
4. TTL -= 1，若歸零則 ICMP Time Exceeded

## 分段與重組

當封包大小超過 MTU 時分段為多個 IP fragment，每個 fragment 包含相同的 Identification 與不同的 Fragment Offset。接收端依 offset 重組原始封包。

## 相關文件

- [route.md](./route.md) — 路由表
- [arp.md](./arp.md) — 位址解析
- [icmp.md](./icmp.md) — ICMP
