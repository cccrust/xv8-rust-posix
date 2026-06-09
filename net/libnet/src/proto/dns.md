# DNS — DNS 協定資料結構

`dns.rs` 定義 DNS 協定的封包結構與序列化/反序列化邏輯。

## DNS 標頭

DNS 訊息以 12 bytes 的固定標頭開頭：

- **ID (16 bits)**: 交易識別碼，匹配請求和回應
- **Flags (16 bits)**: QR（查詢/回應）、Opcode、AA（權威回答）、TC（截斷）、RD（期望遞迴）、RA（支援遞迴）、RCODE（回應碼）
- **QDCOUNT**: Question 區段數量
- **ANCOUNT**: Answer 區段數量
- **NSCOUNT**: Authority 區段數量
- **ARCOUNT**: Additional 區段數量

## 網域名稱壓縮

DNS 使用 label 序列表示域名（如 `3www7example3com0`）。為了節省空間，指向先前出現的域名時使用 2 bytes 的 pointer（高 2 位元為 11）。

## 資源記錄（Resource Record）

Answer、Authority、Additional 區段包含資源記錄：

- NAME: 域名
- TYPE: A (1), AAAA (28), CNAME (5), MX (15), NS (2)
- CLASS: IN (1) for Internet
- TTL: 快取時間（秒）
- RDLENGTH: RDATA 長度
- RDATA: 記錄資料

## 相關文件

- [dns.md](../dns.md) — DNS 協定功能
- [util.md](./util.md) — 協定層工具函式
- [mod.md](./mod.md) — 協定子模組
