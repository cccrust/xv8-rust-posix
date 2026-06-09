# Ping — 網路診斷工具

`ping` 是 xv8 上的網路連通性測試工具，使用 ICMP Echo Request/Reply 機制檢測目標主機是否可達。

## 原理

`ping` 利用 ICMP（RFC 792）的 Type 8（Echo Request）與 Type 0（Echo Reply）封包：

1. 發送 ICMP Echo Request 到目標 IP
2. 等待 Echo Reply
3. 計算 RTT（Round-Trip Time）
4. 若逾時未收到，標記為遺失

## 輸出資訊

- 回應的 ICMP 序號
- RTT（毫秒）
- 封包遺失率（遺失 / 總發送數）
- 統計摘要（最小/最大/平均 RTT）

## 相關文件

- [ping.md](../../kernel/src/net/ping.md) — 核心 ping 實作
- [icmp.md](../../kernel/src/net/icmp.md) — ICMP 協定
- [netping.md](../testbin/netping.md) — 網路 ping 測試
