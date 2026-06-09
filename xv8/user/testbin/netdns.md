# Netdns — DNS 網路測試

`netdns` 測試驗證 xv8 的 DNS 客戶端功能，透過 UDP 發送 DNS 查詢到指定的名稱伺服器，並解析回應。DNS（Domain Name System）是一個階層式分散式命名系統，將網域名稱轉換為 IP 位址。此測試驗證 UDP socket 操作與 DNS 封包解析（A 記錄查詢）在核心協定棧上的正確實作。

## 相關文件

- [dns.md](../../kernel/src/net/dhcp.md) — DHCP/DNS 相關
- [udp.md](../../kernel/src/net/udp.md) — UDP 協定
- [dns.md](../../../net/libnet/src/proto/dns.md) — DNS 協定詳解
