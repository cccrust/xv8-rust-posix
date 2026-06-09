# Ping — ICMP Echo 實作

## 概述

`ping.rs` 實作核心的 ICMP Echo 請求/回覆處理，為使用者空間 ping 工具提供支援。

## Ping 表格

核心維護 ping 連線表格，追蹤待處理的 Echo 請求。每個條目包含 Identifier、Sequence Number、Timestamp（計算 RTT）與 Timeout。

## 處理流程

```
收到 ICMP Type=8（Echo Request）:
  → 交換來源/目標 IP
  → Type 改為 0（Echo Reply）
  → 重新計算 checksum
  → 回傳

收到 ICMP Type=0（Echo Reply）:
  → 根據 Identifier+Sequence 查表
  → 計算 RTT，喚醒等待行程
```

## 相關文件

- [icmp.md](./icmp.md) — ICMP 協定
- [ipv4.md](./ipv4.md) — IP 封裝
