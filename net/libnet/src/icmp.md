# ICMP — 網際網路控制訊息協定

`icmp.rs` 實作 ICMP 協定的客戶端功能，主要用於 `ping` 工具來測試網路連通性。

## ICMP 應用

`libnet` 的 ICMP 模組提供發送 Echo Request 與接收 Echo Reply 的封裝：

1. 計算 ICMP checksum
2. 設定 Identifier 與 Sequence Number
3. 記錄時間戳計算 RTT
4. 處理逾時與封包遺失

## Checksum 計算

ICMP 使用 16-bit 一補數加法檢查碼（Internet checksum）。檢查範圍覆蓋整個 ICMP 封包（含 payload）。計算時 checksum 欄位先填零，計算完成後填入結果。

## 相關文件

- [icmp.md](../proto/icmp.md) — ICMP 資料結構
- [ping.md](../../kernel/src/net/ping.md) — 核心 ping 處理
- [ping.md](../../tools/src/bin/ping.md) — Host ping 工具
