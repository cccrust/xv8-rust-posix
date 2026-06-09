# Traceroute — 路由追蹤工具

`traceroute` 是網路診斷工具，用於追蹤 IP 封包從來源到目的地經過的路徑（路由器）。

## 原理

`traceroute` 利用 TTL（Time To Live）機制與 ICMP Time Exceeded 訊息：

1. 發送 TTL=1 的 UDP 或 ICMP 封包到目標
2. 第一個路由器收到後 TTL 歸零，回傳 ICMP Time Exceeded（Type 11）
3. traceroute 記錄路由器 IP，然後發送 TTL=2 的封包
4. 第二個路由器回傳 Time Exceeded
5. 重複直到封包到達目標（收到 ICMP Port Unreachable 或 Echo Reply）

## 輸出

```
traceroute to 8.8.8.8 (8.8.8.8), 30 hops max
 1  192.168.1.1   1.234 ms
 2  10.0.0.1      2.456 ms
 3  72.14.204.1   5.678 ms
 ...
```

## 相關文件

- [icmp.md](../../kernel/src/net/icmp.md) — ICMP 協定
- [ipv4.md](../../kernel/src/net/ipv4.md) — IP 封包與 TTL
