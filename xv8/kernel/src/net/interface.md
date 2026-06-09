# Interface — 網路介面抽象

## 概述

`interface.rs` 定義了 xv8 網路協定棧的網路介面抽象層，將不同硬體或虛擬裝置統一為共同的程式設計介面。

## NetDevice Trait

```rust
trait NetDevice {
    fn mac_addr(&self) -> MacAddr;
    fn mtu(&self) -> usize;
    fn transmit(&mut self, buf: &[u8]) -> Result<(), NetError>;
    fn receive(&mut self) -> Option<Vec<u8>>;
    fn poll(&mut self) -> bool;
}
```

- **mac_addr()**: 網路卡 MAC 位址
- **mtu()**: 最大傳輸單元
- **transmit()**: 送出封包到硬體鏈路
- **receive()**: 從硬體鏈路讀取封包
- **poll()**: 檢查新封包

## 設計考量

trait 抽象讓 ARP、IP、TCP/UDP 等上層協定無需關心底層硬體。e1000 使用 PCI DMA，loopback 僅記憶體複製，但對 IP 層而言介面一致。

## 相關文件

- [e1000.md](../e1000.md) — e1000 驅動
- [loopback.md](./loopback.md) — Loopback 介面
- [veth.md](./veth.md) — Virtual Ethernet
