# TFTP — 簡易檔案傳輸工具

`tftp` 實作 TFTP（Trivial File Transfer Protocol，RFC 1350）客戶端/伺服器功能。TFTP 使用 UDP 埠 69，是一種極簡的檔案傳輸協定，支援讀取（RRQ）與寫入（WRQ）操作。使用鎖步（lock-step）傳輸模式，每個 DATA 封包需 ACK 確認後才發送下一塊。常用於嵌入式系統的韌體更新。

## 相關文件

- [tftp.md](../../libnet/src/tftp.md) — TFTP 協定實作
- [udp.md](../../kernel/src/net/udp.md) — UDP 傳輸
