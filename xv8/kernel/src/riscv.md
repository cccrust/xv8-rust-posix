# RISC-V 特定功能 — riscv.rs

提供 RISC-V 架構的暫存器存取和控制功能。

## CSR 暫存器模組

```rust
pub mod registers {
    // 機器 ID
    pub mod mhartid { ... }

    // 機器狀態
    pub mod mstatus { ... }

    // 監督模式狀態
    pub mod sstatus { ... }

    // 監督模式陷阱原因
    pub mod scause { ... }

    // 監督模式異常 PC
    pub mod sepc { ... }

    // 監督模式陷阱向量
    pub mod stvec { ... }

    // 計時器
    pub mod time { ... }
    pub mod stimecmp { ... }

    // 頁表
    pub mod satp { ... }

    // 中斷啟用
    pub mod sie { ... }
    pub mod mie { ... }
}
```

## SSTATUS 標誌

```rust
pub const SPP: usize = 1 << 8;   // 上一次特權模式
pub const SPIE: usize = 1 << 5;  // 上一次中斷啟用
pub const SIE: usize = 1 << 1;   // 監督模式中斷啟用
```

## SCAUSE 陷阱類型

```rust
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

pub enum Interrupt {
    UserSoftware,
    SupervisorSoftware,
    UserTimer,
    SupervisorTimer,
    SupervisorExternal,
}

pub enum Exception {
    InstructionAddressMisaligned,
    IllegalInstruction,
    Breakpoint,
    LoadAccessFault,
    StoreAddressMisaligned,
    EnvironmentCall,
    LoadPageFault,
    StorePageFault,
}
```

## 中斷啟用/停用

```rust
pub mod interrupts {
    #[inline]
    pub unsafe fn enable() {
        unsafe { asm!("sret") }  // 透過 sstatus.SPIE 恢復
    }

    #[inline]
    pub unsafe fn disable() {
        unsafe { asm!("csrw sstatus, {}", in(reg) bits) }
    }

    #[inline]
    pub fn get() -> bool {
        sstatus::read() & sstatus::SIE != 0
    }
}
```

## 分頁輔助函數

```rust
// 頁面對齊
pub const fn pg_round_down(x: usize) -> usize { ... }
pub const fn pg_round_up(x: usize) -> usize { ... }

// PTE 處理
pub const PTE_V: usize = 1 << 0;   // Valid
pub const PTE_R: usize = 1 << 1;   // Read
pub const PTE_W: usize = 1 << 2;   // Write
pub const PTE_X: usize = 1 << 3;   // Execute
pub const PTE_U: usize = 1 << 4;   // User
pub const PTE_COW: usize = 1 << 8;  // Copy-on-write

// 計算虛擬位址的頁表索引
pub fn px(level: usize, va: usize) -> usize { ... }

// SATP 格式
pub fn make(ppn: usize) -> usize { ... }
```

## Sv39 分頁

```rust
pub const MAXVA: usize = (1 << (27 * 3 - 1)) - 1;  // 2^38 - 1

// 虛擬位址結構：
// [63:39] - 必須為 0 或 sign-extended
// [38:30] - VPN[2] (一級)
// [29:21] - VPN[1] (二級)
// [20:12] - VPN[0] (三級/葉)
// [11:0]  - 頁內偏移
```

## 上下文切換相關

```rust
pub mod tp {
    // 執行緒指標 = HART ID
    pub unsafe fn write(id: usize) {
        unsafe { asm!("mv tp, {}", in(reg) id) }
    }
}
```

## PMP（實體記憶體保護）

用於在 machine mode 設定記憶體權限：

```rust
pub mod pmpcfg0 { pub unsafe fn write(v: usize) { ... } }
pub mod pmpaddr0 { pub unsafe fn write(v: usize) { ... } }

// 設定為 RWX 對整個實體記憶體
pmpaddr0::write(0x3fffffffffffff);
pmpcfg0::write(0xf);
```

## 定時器

```rust
pub mod time {
    pub unsafe fn read() -> usize {
        unsafe { core::arch::asm!("rdtime {}", out(reg) x) }
    }
}

pub mod stimecmp {
    pub unsafe fn write(t: usize) {
        unsafe { core::arch::asm!("csrw stimecmp, {}", in(reg) t) }
    }
}
```

## TLB 操作

```rust
pub mod vma {
    pub unsafe fn sfence() {
        unsafe { core::arch::asm!("sfence.vma") }
    }
}
```

## 相關主題

- [[Sv39]]：分頁機制
- [[Trap]]：陷阱處理
- [[Boot]]：啟動流程
- [[memlayout]]：記憶體佈局