# Nettools — 網路工具整合測試

`nettools` 測試整合驗證 xv8 的多元件網路功能，同時測試 DNS 查詢、TCP 連線、UDP 資料報與 ICMP Ping 等操作。此測試確保各網路子系統在同行程中協同運作，以及核心網路資源（socket、連接埠）的正確分配與回收。

## 相關文件

- [net.md](./net.md) — 網路基礎測試
- [nettools.md](../../../net/tools/src/bin/tcpclient.md) — 網路工具概覽
- [mod.md](../../kernel/src/net/mod.md) — 協定棧總覽
