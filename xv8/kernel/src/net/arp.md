# ARP — 位址解析協定（RFC 826）

## 概述

ARP（Address Resolution Protocol）負責將 32 位元的 IPv4 位址解析為 48 位元的 Ethernet MAC 位址，運作於 OSI 模型的第二層與第三層之間。

## 協定運作

當核心需要傳送 IP 封包到同一區域網路的主機，但不知道目標 MAC 位址時，會廣播 ARP 請求：

```
乙太網路訊框：目標 MAC = FF:FF:FF:FF:FF:FF（廣播）
ARP 請求：誰擁有 192.168.1.1？請告訴 00:0A:XX:XX:XX:XX
```

擁有該 IP 的主機以 ARP 回覆回應。請求者將對應關係存入 ARP 快取表。

## ARP 封包格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| Hardware Type | 2 bytes | 1 = Ethernet |
| Protocol Type | 2 bytes | 0x0800 = IPv4 |
| HLEN | 1 byte | MAC 位址長度（6） |
| PLEN | 1 byte | IP 位址長度（4） |
| Operation | 2 bytes | 1 = Request, 2 = Reply |
| SHA | 6 bytes | 來源 MAC |
| SPA | 4 bytes | 來源 IP |
| THA | 6 bytes | 目標 MAC（請求時為 0） |
| TPA | 4 bytes | 目標 IP |

## ARP 快取表

ARP 快取將 IP 位址映射到 MAC 位址。每個條目包含狀態（RESOLVED/PENDING/STALE）、TTL 與重試計數。PENDING 條目會定期重新發送請求。

## 相關文件

- [eth.md](./eth.md) — Ethernet II 訊框
- [ipv4.md](./ipv4.md) — IPv4 封包格式
- [interface.md](./interface.md) — 網路介面抽象
