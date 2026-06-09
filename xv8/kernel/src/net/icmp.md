# ICMP — 網際網路控制訊息協定（RFC 792）

## 概述

ICMP（Internet Control Message Protocol）是 IP 的附屬協定，用於傳遞錯誤報告與診斷資訊。ICMP 封包封裝在 IP 中，但被視為 IP 層的一部分。

## ICMP 封包格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| Type | 1 byte | 訊息類型 |
| Code | 1 byte | 細分類別 |
| Checksum | 2 bytes | 16-bit 補數和 |
| Rest of Header | 4 bytes | 依類型而異 |
| Payload | 可變 | 附加資料 |

## 常見類型

| Type | 名稱 | 用途 |
|------|------|------|
| 0 | Echo Reply | ping 回覆 |
| 3 | Destination Unreachable | 無法送達 |
| 8 | Echo Request | ping 請求 |
| 11 | Time Exceeded | TTL 歸零（traceroute） |

## Echo Request/Reply

`ping` 的核心機制：發送 Type=8 請求，接收端回覆 Type=0 回覆。封包包含 Identifier 與 Sequence Number 用於匹配。

## 相關文件

- [ipv4.md](./ipv4.md) — IP 封裝
- [ping.md](./ping.md) — Ping 實作
