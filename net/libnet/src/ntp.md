# NTP — 網路時間協定

`ntp.rs` 實作 NTP（Network Time Protocol）客戶端功能，用於從 NTP 伺服器取得精確的系統時間。

## NTP 協定基礎

NTP（RFC 5905）用於同步電腦系統時間，使用階層式架構：

- **Stratum 0**: 原子鐘、GPS 等高精度時間源
- **Stratum 1**: 直接與 Stratum 0 同步的時間伺服器
- **Stratum 2**: 與 Stratum 1 同步的伺服器（依此類推）

## NTP 封包格式

NTP 使用 UDP 埠 123。關鍵欄位包含：

- **Reference Timestamp**: 系統最後一次同步的時間
- **Originate Timestamp**: 客戶端傳送請求的時間
- **Receive Timestamp**: 伺服器收到請求的時間
- **Transmit Timestamp**: 伺服器傳送回應的時間

偏移量計算：`offset = ((T2 - T1) + (T3 - T4)) / 2`

## 相關文件

- [ntp.md](../../tools/src/bin/ntp.md) — NTP 工具
- [udp.md](../../kernel/src/net/udp.md) — UDP 傳輸
- [time.md](../../../xv8rust/xv8-user-std/src/time.md) — 時間操作
