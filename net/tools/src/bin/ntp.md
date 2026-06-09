# NTP — 網路時間協定客戶端

`ntp` 是一個 NTP 時間同步客戶端，從 NTP 伺服器取得精確的時間資訊。NTP（RFC 5905）使用階層式時間源架構，透過 UDP 埠 123 交換時間戳。計算偏移量後校正本地時間。此工具輸出伺服器時間、本地時間與兩者的差異。

## 相關文件

- [ntp.md](../../libnet/src/ntp.md) — NTP 協定實作
- [time.md](../../../xv8rust/xv8-user-std/src/time.md) — 時間操作
