# Route — 路由表

## 概述

路由表根據目標 IP 位址決定封包應送往哪個網路介面及下一跳閘道器。

## 最長前綴匹配

路由決策的核心原則是 LPM（Longest Prefix Match）：找到與目標 IP 前綴匹配長度最長的條目。

## 路由條目

| 欄位 | 說明 |
|------|------|
| Destination | 目標網路 |
| Netmask | 子網路遮罩 |
| Gateway | 下一跳閘道器（直連則為 0.0.0.0） |
| Interface | 輸出介面 |
| Metric | 路由成本 |

## 預設路由

`0.0.0.0/0` 是預設路由。當目標 IP 不匹配任何更特定路由時，使用預設閘道器轉發。

## 相關文件

- [ipv4.md](./ipv4.md) — IP 層
- [arp.md](./arp.md) — 位址解析
- [interface.md](./interface.md) — 網路介面
