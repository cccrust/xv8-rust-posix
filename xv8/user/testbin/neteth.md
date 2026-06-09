# Neteth — Ethernet 層測試

`neteth` 測試驗證 xv8 的 Ethernet 訊框處理功能，確認乙太網路標頭的正確封裝/解封裝、ARP 位址解析以及 MTU 處理。Ethernet 是區域網路的基礎資料連結層技術，遵循 IEEE 802.3 標準。此測試從應用層到 Ethernet 晶片驅動，驗證整條資料傳輸鏈。

## 相關文件

- [eth.md](../../kernel/src/net/eth.md) — Ethernet 訊框
- [arp.md](../../kernel/src/net/arp.md) — ARP 解析
- [interface.md](../../kernel/src/net/interface.md) — 網路介面
