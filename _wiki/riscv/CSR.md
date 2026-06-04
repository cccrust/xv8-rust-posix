# CSR — 控制與狀態暫存器

RISC-V 使用 CSR 來控制處理器行為和報告狀態。

## CSR 列表

xv8 使用的主要 CSR：

| 簡寫 | 全名 | 用途 |
|------|------|------|
| `mstatus` | Machine Status | 全域中斷開關、特權模式 |
| `mie` | Machine Interrupt Enable | 機器模式中斷啟用 |
| `mip` | Machine Interrupt Pending | 待處理中斷 |
| `mtvec` | Machine Trap Vector | 機器模式 trap 向量 |
| `mepc` | Machine Exception PC | 機器模式例外 PC |
| `mcause` | Machine Cause | 例外/中斷原因 |
| `mscratch` | Machine Scratch | 暫存器保存 |
| `sstatus` | Supervisor Status | S 模式狀態 |
| `stvec` | Supervisor Trap Vector | S 模式 trap 向量 |
| `sepc` | Supervisor Exception PC | S 模式例外 PC |
| `scause` | Supervisor Cause | S 模式原因 |
| `stval` | Supervisor Trap Value | 附加錯誤資訊 |
| `satp` | Supervisor Address Translation | 頁表指標 |
| `sie` | Supervisor Interrupt Enable | S 模式中斷啟用 |

## mstatus — 機器狀態

```rust
pub const MPP_MASK: usize = 3 << 11;
pub const MPP_SUPERVISOR: usize = 1;
```

- **MPP[11:10]**：先前特權模式（回傳時用）
- **SIE[1]**：監督者中斷啟用
- **SPIE[5]**：先前 SIE（trap 前的值）
- **SPP[8]**：先前 U/S 模式

```rust
// 讀取 mstatus
let bits: usize;
asm!("csrr {}, mstatus", out(reg) bits);

// 寫入 mstatus
asm!("csrw mstatus, {}", in(reg) bits);
```

## sstatus — 監督者狀態

mstatus 的子集，適用於 S 模式：

```rust
pub const SPP: usize = 1 << 8;   // 先前模式
pub const SPIE: usize = 1 << 5;  // 先前中斷啟用
pub const SIE: usize = 1 << 1;   // 中斷啟用
```

## scause — Trap 原因

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
    LoadAccessFault,
    StoreAddressMisaligned,
    EnvironmentCall,      // ecall
    LoadPageFault,
    StorePageFault,
}
```

高位表示是中斷（1）還是例外（0）：
```rust
pub fn is_interrupt(&self) -> bool {
    self.bits() & (1 << (usize::BITS as usize - 1)) != 0
}
```

## satp — 頁表指標

```rust
const SV39: usize = 8 << 60;  // Sv39 模式

pub fn make(pagetable: usize) -> usize {
    SV39 | (pagetable >> 12)  // PPN of root page table
}
```

格式：
```
  63    60    0
┌──────┬──────┐
│ MODE │ PPN │
└──────┴──────┘
```

MODE = 8 表示 Sv39。

## stvec — Trap 向量

設定 trap 處理常式位址：
- **MODE = 0**：所有 trap 到同一個位址
- **MODE = 1**：向量模式（不同中斷到不同偏移）

```rust
pub mod stvec {
    pub unsafe fn write(bits: usize) {
        asm!("csrw stvec, {}", in(reg) bits);
    }
}
```

## sie / sie — 中斷啟用

```rust
pub const SEIE: usize = 1 << 9;  // 外部中斷
pub const STIE: usize = 1 << 5;  // 計時器中斷
pub const SSIE: usize = 1 << 1;  // 軟體中斷
```

## 存取語法

```rust
// 讀取 CSR
asm!("csrr {}, csr_name", out(reg) result);

// 寫入 CSR
asm!("csrw csr_name, {}", in(reg) value);

// 讀後修改
asm!("csrrc {}, csr_name, {}", out(reg) old, in(reg) mask);
// 或
asm!("csrrs {}, csr_name, {}", out(reg) old, in(reg) mask);
```

## mret / sret / uret

從 trap 返回：
- **mret**：從 M 模式返回
- **sret**：從 S 模式返回
- **uret**：從 U 模式返回

返回時：
1. 從 `*epc` 恢復 PC
2. 從 `*status` 恢復中斷狀態
3. 切換回先前的特權模式

```rust
asm!("mret", options(noreturn));
```

## medeleg / mideleg — 例外委派

將特定例外/中斷委派給 S 模式處理：

```rust
medeleg::write(0xffff);  // 委派所有例外
mideleg::write(0xffff);  // 委派所有中斷
```

這樣 M 模式收到這些 trap 後會直接轉給 S 模式。

## 本專案使用

xv8 核心在 `kernel/src/riscv.rs` 定義了所有 CSR 操作：

```rust
pub mod registers {
    pub mod mstatus { ... }
    pub mod sstatus { ... }
    pub mod scause { ... }
    pub mod satp { ... }
    pub mod sie { ... }
    pub mod stvec { ... }
    // ...
}
```

## 與 x86 的對比

| RISC-V CSR | x86 MSR | 用途 |
|------------|---------|------|
| mstatus | RFLAGS | 狀態旗標 |
| sstatus | CR0 | 控制暫存器 |
| satp | CR3 | 頁表指標 |
| stvec | IDT base | trap 向量 |
| scause | vector | 例外編號 |