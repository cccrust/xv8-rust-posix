# Eth — 乙太網路訊框（RFC 894）

## 概述

乙太網路是區域網路中最廣泛使用的資料連結層技術，定義了裝置間經由實體媒體傳輸資料的格式。

## Ethernet II 訊框格式

| 欄位 | 長度 | 說明 |
|------|------|------|
| 目標 MAC | 6 bytes | 接收端 MAC 位址 |
| 來源 MAC | 6 bytes | 發送端 MAC 位址 |
| EtherType | 2 bytes | 上層協定：0x0800=IPv4, 0x0806=ARP |
| 承載 | 46-1500B | LLC 或上層協定資料 |
| FCS | 4 bytes | Frame Check Sequence（CRC32） |

## MAC 位址

48 位元硬體位址，前 24 位元為 OUI（廠商代碼），後 24 位元由廠商分配。廣播位址為 `FF:FF:FF:FF:FF:FF`。

## MTU

Ethernet MTU 為 1500 bytes，限制單一訊框可承載的上層協定資料量。超過 MTU 的封包需由 IP 層分段。

## 相關文件

- [arp.md](./arp.md) — ARP 協定
- [ipv4.md](./ipv4.md) — IPv4 封裝
- [interface.md](./interface.md) — 網路介面
