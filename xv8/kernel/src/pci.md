# PCI 匯流排 — pci.rs

PCI（Peripheral Component Interconnect）用於枚舉和設定系統中的 PCIe/MMIO 裝置。

## QEMU PCI 配置

```
PCI ECAM (Enhanced Configuration Access Mechanism)
0x3000_0000 ─────────────────────────────────────
│ Bus 0                                              │
│     ├─ Device 1: VirtIO Block (無 BAR0)          │
│     └─ Device 2: E1000 (0x100E, BAR0=0x40000000)│
│     └─ Device 3: ...
```

## 組織結構

```
位址 = PCI_ECAM + (bus << 20) + (device << 15) + (function << 12)
                8 bits           5 bits           3 bits
```

## 配置空間

每個 PCI 函數有 4KB 配置空間：

```
Offset   內容
0x00     Vendor ID, Device ID
0x04     Command, Status
0x08     Class Code, Revision ID
0x0C     BIST, Header Type, Latency Timer, Cache Line Size
0x10     BAR0 (Base Address Register 0)
...
0x3C     Max Latency, Min Grant, Int PIN, Int LINE
```

## BAR（Base Address Register）

用於查詢和設定 MMIO 或 I/O 空間：

```rust
unsafe fn setup_bar0(bar0_ptr: *mut u32) -> Option<u64> {
    // 寫入全 1 以讀取大小請求
    ptr::write_volatile(bar0_lo_ptr, 0xFFFF_FFFF);
    let bar0_lo = ptr::read_volatile(bar0_lo_ptr);

    if bar0_lo == 0 || bar0_lo & 1 == 1 {
        return None;  // 未實現或 I/O 空間
    }

    let is_64bit = bar0_lo & 0x6 == 0x4;

    let bar0_hi = if is_64bit {
        ptr::read_volatile(bar0_hi_ptr)
    } else {
        0
    };

    let mask = bar0_lo & !0b1111;
    // 計算對齊後的位址
}
```

## 枚舉過程

```rust
pub unsafe fn init() {
    for bus in 0..256 {
        for device in 0..32 {
            let addr = get_ecam_offset(bus, device, 0) as *const u32;

            let vendor_id = ptr::read_volatile(addr) as u16;

            if vendor_id == 0xFFFF {
                continue;  // 無裝置
            }

            // 設定 BAR0 並獲取 MMIO 位址
            let Some(bar_addr) = setup_bar0(addr.add(1) as *mut u32) else {
                continue;
            };

            // 啟用 Bus Master 和 Memory Space
            let cmd = ptr::read_volatile(addr.add(1) as *const u16);
            ptr::write_volatile(addr.add(1) as *mut u16, cmd | (1 << 2) | (1 << 1));

            // 記錄 E1000
            if vendor_id == 0x8086 && device_id == 0x100E {
                E1000_BASE.store(bar_addr as usize, Ordering::SeqCst);
            }
        }
    }
}
```

## 已知裝置

| Vendor | Device | 名稱 | BAR0 |
|--------|--------|------|------|
| 0x1AF4 | 0x1002 | VirtIO Block | N/A |
| 0x8086 | 0x100E | E1000 | 0x40000000 |

## MMIO 配置

```rust
static MMIO_NEXT: SpinLock<usize> = SpinLock::new(PCI_MMIO, "pci_mmio");

// 動態分配 MMIO 區域
let aligned = (*cursor as u64 + !mask) & mask;
let size = !mask + 1;
*cursor = (aligned + size) as usize;
```

## E1000 整合

```rust
pub static E1000_BASE: AtomicUsize = AtomicUsize::new(0);

// 在 pci::init() 中
if vendor_id == 0x8086 && device_id == 0x100E {
    E1000_BASE.store(bar_addr as usize, Ordering::SeqCst);
}

// 在 e1000::init() 中
let base = E1000_BASE.load(Ordering::SeqCst);
```

## 相關主題

- [[e1000]]：網卡驅動
- [[memlayout]]：記憶體佈局
- [[plic]]：中斷控制器