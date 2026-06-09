# Lib — 協定函式庫

`lib.rs` 是 `libnet` crate 的根模組，提供用於網路工具開發的協定實作庫。

## 設計目的

`libnet` 將網路協定的通用元件抽取為可重用函式庫，避免在 ping、dns、ntp、tftp 等工具中重複實作相同的協定處理邏輯。

## 提供的功能

- **dns**: DNS 查詢/回應的序列化與反序列化
- **icmp**: ICMP 封包封裝/解封裝
- **ntp**: NTP 時間協定客戶端
- **tftp**: TFTP 檔案傳輸協定
- **net_impl**: 跨平台的網路 socket 操作包裝
- **util**: 位址解析、位元組操作等工具函式

## 相關文件

- [net_impl.md](./net_impl.md) — 網路操作實作
- [util.md](./util.md) — 共用工具函式
- [dns.md](./dns.md) — DNS 協定
- [icmp.md](./icmp.md) — ICMP 協定
