# ICMP — ICMP 資料結構

`icmp.rs` 定義 ICMP（Internet Control Message Protocol）封包的資料結構。

## ICMP 封包結構

```rust
struct IcmpHeader {
    typ: u8,       // 類型
    code: u8,      // 代碼
    checksum: u16, // 檢查碼
    // 接下來依類型而異
}

struct IcmpEcho {
    id: u16,        // 識別碼
    sequence: u16,  // 序號
}
```

## 支援的類型

`proto/icmp.rs` 定義了以下 ICMP 類型的結構：

- **Echo Request (8, 0)**: 包含 Identifier + Sequence Number + 時間戳（payload）
- **Echo Reply (0, 0)**: 與 Echo Request 相同結構
- **Destination Unreachable (3, 0-15)**: 包含原始 IP 封包的前 8 bytes
- **Time Exceeded (11, 0-1)**: 包含原始 IP 封包的前 8 bytes

## 序列化/反序列化

資料結構支援從位元組緩衝區中解析（parse）與序列化（serialize）為網路位元組序（大端序）。

## 相關文件

- [icmp.md](../icmp.md) — ICMP 功能實作
- [mod.md](./mod.md) — 協定子模組
