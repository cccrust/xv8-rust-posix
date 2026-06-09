# Netping — Ping 網路測試

`netping` 測試驗證 xv8 的 ICMP Echo 功能，透過發送 ping 請求並接收回應來測量網路連通性與 RTT（Round-Trip Time）。測試驗證 ICMP 封包的產生、封裝、傳送、接收與解析的完整流程，以及核心 ping 表格的正確管理（匹配請求和回覆）。

## 相關文件

- [ping.md](../../kernel/src/net/ping.md) — 核心 ping 實作
- [icmp.md](../../kernel/src/net/icmp.md) — ICMP 協定
- [ipv4.md](../../kernel/src/net/ipv4.md) — IP 封裝
