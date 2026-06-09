# DNS — 網域名稱系統協定

`dns.rs` 實作 DNS 協定客戶端功能，支援將網域名稱解析為 IP 位址。

## DNS 協定基礎

DNS（Domain Name System）是網際網路的分散式目錄服務，將人類可讀的網域名稱轉換為機器可讀的 IP 位址。查詢使用 UDP 埠 53（若回應被截斷則回退至 TCP 埠 53）。

## 查詢類型

- **A 記錄 (type 1)**: 將域名映射到 IPv4 位址
- **AAAA 記錄 (type 28)**: 將域名映射到 IPv6 位址
- **CNAME 記錄 (type 5)**: 域名別名
- **MX 記錄 (type 15)**: 郵件交換伺服器
- **NS 記錄 (type 2)**: 域名伺服器

## DNS 封包結構

DNS 訊息由五個區段組成：Header（12 bytes）、Question、Answer、Authority、Additional。使用網域名稱壓縮（pointer 指向先前出現的字串）減少封包大小。

## 相關文件

- [dns.md](../proto/dns.md) — DNS 封包結構
- [udp.md](../../kernel/src/net/udp.md) — UDP 傳輸
- [net_impl.md](./net_impl.md) — 網路工具實作
