# Interrupts — 中斷處理

RISC-V 的中斷處理機制，讓核心能回應計時器和外部裝置。

## 中斷類型

| 類型 | 簡寫 | 來源 |
|------|------|------|
| 軟體中斷 | SI/SSI | 其他 HART 觸發 |
| 計時器中斷 | TI/STI | 計時器硬體 |
| 外部中斷 | EI/SEI | UART、VirtIO、網卡等 |

## 中断框架

```
裝置 → PLIC → sip SEIP → sie SEIE → sstatus SIE → trap
                        │                           │
                        └──────── 中斷───────────────┘
```

## 軟體中斷

用於 HART 間通訊：

```rust
// 啟用
sie::write(sie::read() | sie::SSIE);

// 觸發（從另一個 HART）
// 寫入 clint 的 MSIP 暫存器
```

## 計時器中斷

```rust
pub const STIE: usize = 1 << 5;

unsafe fn timer_init() {
    // 啟用 S 模式計時器中斷
    mie::write(mie::read() | mie::STIE);

    // 設定下次計時器中斷
    let next = time::read() + 1000000;  // ~1ms
    stimecmp::write(next);
}
```

### 計時器中斷處理

```rust
pub fn usertrap() {
    let cause = scause::read().cause();
    match cause {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 設定下次計時器中斷
            let next = time::read() + 1000000;
            unsafe { stimecmp::write(next); }
            // 讓出 CPU
            yield_cpu();
        }
        // ...
    }
}
```

## 外部中斷 (PLIC)

外部裝置（UART、VirtIO、E1000）通過 PLIC 中斷。

### PLIC 結構

```
PLIC 暫存器 (0x0c000000):
- 0x0000: priority base
- 0x1000: pending base
- 0x2000: enable base (per hart)
- 0x200000: claim/complete base (per hart)
```

### PLIC 初始化

```rust
pub const PLIC: usize = 0x0C00_0000;

pub const fn PLIC_SENABLE(hart: usize) -> u32 {
    (PLIC + 0x2080 + (hart * 0x100)) as u32
}

pub const fn PLIC_SCLAIM(hart: usize) -> u32 {
    (PLIC + 0x201004 + (hart * 0x2000)) as u32
}
```

### PLIC 中斷處理

```rust
fn devintr() -> bool {
    let scause = scause::read();

    if scause.is_interrupt() {
        match Interrupt::from(scause.code()) {
            Interrupt::SupervisorExternal => {
                // 從 PLIC 取得中斷號
                let claim = PLIC_SCLAIM(hart).read();

                if claim == UART0_IRQ {
                    uartintr();  // UART 中斷
                } else if claim == VIRTIO0_IRQ {
                    virtio_disk_intr();  // 磁碟中斷
                } else if claim == E1000_IRQ {
                    e1000_intr();  // 網卡中斷
                }

                // 通知 PLIC 中斷已處理
                PLIC_SCLAIM(hart).write(claim);
                true
            }
            _ => false,
        }
    } else {
        false
    }
}
```

## 中斷啟用/停用

### 全域啟用/停用

```rust
pub fn enable() {
    unsafe { sstatus::write(sstatus::read() | sstatus::SIE) };
}

pub fn disable() {
    unsafe { sstatus::write(sstatus::read() & !sstatus::SIE) };
}

pub fn get() -> bool {
    unsafe { (sstatus::read() & sstatus::SIE) != 0 }
}
```

### per-HART 啟用

```rust
// 啟用特定中斷類型
mie::write(mie::read() | mie::STIE);  // 計時器
mie::write(mie::read() | mie::SEIE);  // 外部
mie::write(mie::read() | mie::SSIE);  // 軟體
```

## 中斷優先級

PLIC 支援中斷優先級（但 xv8 未使用）：

```rust
// 設定優先級
fn plic_set_priority(irq: u32, priority: u32) {
    (PLIC + irq * 4) as *mut u32).write_volatile(priority);
}
```

## 巢狀中斷

RISC-V 不支援硬體巢狀中斷。軟體必須：
1. 停用中斷
2. 保存狀態
3. 處理中斷
4. 恢復狀態
5. 重新啟用中斷

```rust
pub fn usertrap() {
    disable();  // 停用中斷

    // 處理...

    enable();   // 重新啟用
    sret();     // 返回
}
```

## 與 x86 APIC 的比較

| 特性 | RISC-V PLIC | x86 APIC |
|------|-------------|----------|
| 架構 | 集中式 | 分散式（Local APIC + I/O APIC）|
| 中斷數 | 取決於實作 | 最多 24 個 |
| 優先級 | 可設定 | 可設定 |
| MSI | 不支援（v0.10 支援）| 支援 |

## 計時器中斷觸發流程

```
1. time::read() >= stimecmp::write()
         │
         ▼
2. 計時器硬體設定 sip STIP
         │
         ▼
3. 如果 sie STIE 且 sstatus SIE
         │
         ▼
4. trap 進入 usertrap()
         │
         ▼
5. 處理計時器中斷
         │
         ▼
6. 設定下次 stimecmp
         │
         ▼
7. sret 返回
```

## 中斷延遲

中斷延遲取決於：
- 計時器精度
- PLIC 路由延遲
- 軟體處理時間

QEMU 環境約 1-10ms。

## 性能考量

- 中斷處理需要儲存/恢復暫存器
- 頻繁中斷影響效能
- 使用屏障減少中斷頻率