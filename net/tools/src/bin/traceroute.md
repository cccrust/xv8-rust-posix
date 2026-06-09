# Traceroute — 路由追蹤工具

`traceroute` 追蹤 IP 封包從本機到目標主機的路徑。它利用 TTL（Time To Live）機制與 ICMP Time Exceeded 訊息，逐跳探測路徑上的路由器。每次探測發送 TTL 遞增的封包（UDP 或 ICMP Echo），收集每個路由器的 IP 位址與 RTT。用於診斷網路路由問題與延遲瓶頸。

## 相關文件

- [ping.md](./ping.md) — Ping 工具
- [icmp.md](../../kernel/src/net/icmp.md) — ICMP 協定
- [ipv4.md](../../kernel/src/net/ipv4.md) — IP 封包與 TTL
