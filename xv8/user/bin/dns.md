# DNS — 網域名稱查詢工具

`dns` 是在 xv8 上執行的 DNS（Domain Name System）查詢工具。DNS 是網際網路的目錄服務，將人類可讀的網域名稱（如 `example.com`）轉換為 IP 位址。

## DNS 查詢流程

1. 使用者輸入網域名稱
2. 程式向設定的 DNS 伺服器（通常為 `8.8.8.8` 或 DHCP 取得的位址）發送 UDP 查詢
3. DNS 伺服器回傳 A 記錄（IPv4）或 AAAA 記錄（IPv6）
4. 程式顯示解析結果

DNS 查詢使用 UDP 埠 53，若回應被截斷則回退至 TCP 埠 53。xv8 的 `dns` 工具支援最基本的 A 記錄查詢。

## 相關文件

- [netdns.md](../testbin/netdns.md) — DNS 網路測試
- [dns.md](../../../net/libnet/src/proto/dns.md) — DNS 協定資料結構
