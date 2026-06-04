# 記憶體佈局 — memlayout.rs

定義 xv8 在 QEMU virt 機器上的記憶體映射。

## QEMU 虛擬機記憶體映射

```
位址範圍            用途
─────────────────────────────────────────
0x0000_1000         BOOT ROM (QEMU 提供)
0x0200_0000         CLINT (時鐘中斷)
0x0C00_0000         PLIC (中斷控制器)
0x1000_0000         UART0 (序列埠)
0x1000_1000         VIRTIO DISK
0x3000_0000         PCI ECAM (配置空間)
0x4000_0000         PCI MMIO
0x8000_0000         核心載入位址 (預設)
```

## 核心記憶體佈局

```
0x8000_0000 ────── KERNBASE (核心載入基址)
    │
    ├── 核心文字段 (text, rodata)
    │
    ├── 核心資料段 (data, bss)
    │
end ─────────────── 核心配置器起始
    │
    ├── 實體頁配置 (Buddy 配置器)
    │
    │
0x8800_0000 ────── PHYSTOP (128MB)
```

## 虛擬位址空間（ Sv39）

```
使用者空間：
0x0000_0000 ────── 0
    │
    ├── text
    ├── data
    ├── heap (向上增長)
    │
    ├── mmap 區域 (向下增長)
    │
TRAPFRAME ────────── TRAPFRAME (1 頁)
TRAMPOLINE ───────── TRAMPOLINE (1 頁)

核心空間：
TRAMPOLINE - 1 ──── 最高核心虛擬位址
    │
    ├── 核心堆疊 (每程序)
    │
    │
KERNBASE ──────────── 核心基址
```

## 關鍵常數

```rust
// 序列埠
pub const UART0: usize = 0x1000_0000;
pub const UART0_IRQ: usize = 10;

// VirtIO 磁碟
pub const VIRTIO0: usize = 0x1000_1000;
pub const VIRTIO0_IRQ: usize = 1;

// PLIC
pub const PLIC: usize = 0x0C00_0000;

// PCI
pub const PCI_ECAM: usize = 0x3000_0000;  // 256 MB
pub const PCI_MMIO: usize = 0x4000_0000;  // 1 GB

// E1000 網卡
pub const E1000_IRQ: usize = 33;

// 實體記憶體
pub const KERNBASE: usize = 0x8000_0000;
pub const PHYSTOP: usize = KERNBASE + (128 * 1024 * 1024);  // 128 MB
```

## Trampoline 頁面

```rust
pub const TRAMPOLINE: usize = MAXVA - PGSIZE;
```

Trampoline 頁面映射到使用者和管理空間的最高位址，用於陷阱進入/退出。

## 核心堆疊配置

```rust
pub const fn kstack(p: usize) -> usize {
    TRAMPOLINE - (p + 1) * ((NKSTACK_PAGES + 1) * PGSIZE)
}
```

每個程序的堆疊周圍有無效守衛頁，防止堆疊溢位。

## 使用者/核心切換

```
使用者空間 → TRAMPOLINE → 核心空間
    │
    ├── trampoline.S::uservec
    ├── 儲存使用者暫存器到 trapframe
    ├── 切換頁表
    └── 跳轉到 usertrap
```

## PTE 標誌含義

```
[0] V - Valid (有效)
[1] R - Read (可讀)
[2] W - Write (可寫)
[3] X - Execute (可執行)
[4] U - User (使用者可用)
[5] G - Global (全域映射)
[6] A - Accessed (已訪問)
[7] D - Dirty (髒頁)
[8] C - Copy-On-Write (複製寫入)
[10:9] RSW - 保留給監督軟體
```

## 相關主題

- [[Sv39]]：RISC-V 分頁機制
- [[vm]]：虛擬記憶體
- [[Trap]]：陷阱處理
- [[Boot]]：啟動流程