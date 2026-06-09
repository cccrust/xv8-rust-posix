# Net — 網路功能測試

`net` 測試驗證 xv8 核心網路協定棧的基礎功能，包括 socket 建立、TCP 連線、UDP 資料報傳送、bind/listen/accept 操作。測試涵蓋從 socket 系統呼叫到實體網路卡驅動的完整路徑，驗證封包經由 IP → ARP → Ethernet → e1000 的傳輸鏈正常運作。

## 相關文件

- [neteth.md](./neteth.md) — Ethernet 層測試
- [netdns.md](./netdns.md) — DNS 網路測試
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
- [mod.md](../../kernel/src/net/mod.md) — 協定棧總覽
