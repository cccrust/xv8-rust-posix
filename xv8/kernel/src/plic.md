# PLIC 中斷控制器 — plic.rs

PLIC（Platform Level Interrupt Controller）是 RISC-V 的標準中斷控制器。

## QEMU 中的 PLIC

```
PLIC (0x0C000000)
         │
    ┌────┴────┐
    │         │
  Hart 0    Hart 1
  S-mode    S-mode
         │
    ┌────┴────┐
    │         │
 UART0    VIRTIO
 IRQ 10   IRQ 1
```

## 寄存器映射

```rust
pub const PLIC: usize = 0x0C00_0000;

pub const fn PLIC_SENABLE(hart: usize) -> u32 {
    (PLIC + 0x2080 + (hart * 0x100)) as u32  // 中斷啟用
}

pub const fn PLIC_SPRIORITY(hart: usize) -> u32 {
    (PLIC + 0x201000 + (hart * 0x2000)) as u32  // 優先級
}

pub const fn PLIC_SCLAIM(hart: usize) -> u32 {
    (PLIC + 0x201004 + (hart * 0x2000)) as u32  // 宣告/完成
}
```

## 中斷優先級

每個中斷有 0-7 的優先級（0 表示停用）：

```rust
unsafe fn init() {
    // 設定優先級（非零表示啟用）
    ptr::write_volatile((PLIC + (UART0_IRQ * 4)) as *mut u32, 1);
    ptr::write_volatile((PLIC + (VIRTIO0_IRQ * 4)) as *mut u32, 1);
    ptr::write_volatile((PLIC + (E1000_IRQ * 4)) as *mut u32, 1);
}
```

## 中斷啟用

每個 hart 有獨立的啟用暫存器：

```rust
pub unsafe fn init_hart() {
    let hart = proc::current_id();

    // 啟用 UART0 (IRQ 10) 和 VIRTIO0 (IRQ 1)
    // IRQ 0-31 使用 word 0
    ptr::write_volatile(
        PLIC_SENABLE(hart) as *mut u32,
        (1 << UART0_IRQ) | (1 << VIRTIO0_IRQ),
    );

    // E1000 (IRQ 33) 使用 word 1 (IRQ 32-63)
    ptr::write_volatile(
        (PLIC_SENABLE(hart) as *mut u32).add(1),
        1 << (E1000_IRQ - 32),
    );
}
```

## 宣告中斷

```rust
pub fn claim() -> u32 {
    let hart = proc::current_id();
    // 讀取並返回待處理的最高優先級中斷
    ptr::read_volatile(PLIC_SCLAIM(hart) as *const u32)
}
```

## 完成中斷

```rust
pub fn complete(irq: u32) {
    let hart = proc::current_id();
    // 通知 PLIC 中斷已處理
    ptr::write_volatile(PLIC_SCLAIM(hart) as *mut u32, irq);
}
```

## 處理流程

```rust
fn device_interrupt(intr: scause::Interrupt) -> Option<InterruptType> {
    match intr {
        scause::Interrupt::SupervisorExternal => {
            let irq = plic::claim();

            match irq {
                0 => {} // 偽造中斷
                UART0_IRQ => uart::handle_interrupt(),
                VIRTIO0_IRQ => virtio_disk::handle_interrupt(),
                E1000_IRQ => e1000::handle_interrupt(),
                _ => println!("unexpected irq = {}", irq),
            }

            plic::complete(irq);
            Some(InterruptType::Device)
        }
        // ...
    }
}
```

## 優先級閾值

```rust
// 設定閾值為 0（處理所有優先級 > 0 的中斷）
ptr::write_volatile(PLIC_SPRIORITY(hart) as *mut u32, 0);
```

## 與 trap.rs 的整合

```
外部中斷觸發
    │
    ▼
scause::Interrupt::SupervisorExternal
    │
    ▼
plic::claim() ──────────────────────────→ PLIC
    │                                         │
    │                                         ▼
    │                                    返回最高優先級 IRQ
    │                                         │
    │◄────────────────────────────────────────┘
    │
    ▼
分派到具體驅動
    │
    ▼
plic::complete(irq)
```

## 相關主題

- [[trap]]：陷阱處理
- [[uart]]：序列埠
- [[virtio_disk]]：磁碟
- [[e1000]]：網卡