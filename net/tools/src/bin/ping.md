# Ping — 網路連通測試

`ping` 使用 ICMP Echo Request/Reply 測試主機可達性。它發送 ICMP Echo Request 封包到目標，等待 Echo Reply，計算 RTT（Round-Trip Time）與封包遺失率。Ping 是診斷網路連線問題的首選工具，可檢測網路延遲、封包遺失與路由問題。

## 相關文件

- [icmp.md](../../libnet/src/icmp.md) — ICMP 實作
- [icmp.md](../proto/icmp.md) — ICMP 資料結構
- [ping.md](../../kernel/src/net/ping.md) — 核心 ping 處理
