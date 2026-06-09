# DNS — 名稱解析工具

`dns` 是一個命令列 DNS 查詢工具，用於查詢網域名稱的 DNS 記錄。它支援查詢 A（IPv4）、AAAA（IPv6）、CNAME、MX、NS 等多種記錄類型。使用者可指定目標 DNS 伺服器。DNS 是網際網路的分散式命名系統，透過 UDP（必要時回退至 TCP）埠 53 進行查詢。

## 相關文件

- [host.md](./host.md) — Host 名稱查詢
- [dns.md](../../libnet/src/proto/dns.md) — DNS 協定結構
