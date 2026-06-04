# PMP — 實體記憶體保護

Physical Memory Protection (PMP) 是 RISC-V 的記憶體存取控制機制。

## 概述

PMP 允許 M 模式設定記憶體區域的存取權限，限制 S/U 模式的存取。

## PMP 暫存器

| 暫存器 | 說明 |
|--------|------|
| `pmpcfg0` - `pmpcfg3` | 配置（4 個實體記憶體保護區域）|
| `pmpaddr0` - `pmpaddr15` | 區域位址（最多 16 個區域）|

xv8 只使用第一個區域（pmpcfg0 + pmpaddr0）。

## pmpaddr 格式

```rust
pub const ADDR_MASK: usize = (1 << 54) - 1;  // 只使用低 54 位元
```

位址右移 2 位元（因為最小顆粒是 4 bytes）：
```rust
pub unsafe fn write(bits: usize) {
    asm!("csrw pmpaddr0, {}", in(reg) bits);
}
```

## pmpcfg0 格式

每個區域 8 bits：

```
  7      6     5      4      3      2      1      0
┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
│  L   │  0   │  0   │  0   │  A   │  X   │  W   │  R   │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
```

- **R**：讀取權限
- **W**：寫入權限
- **X**：執行權限
- **A**：位址匹配模式
- **L**：鎖定（對 M 模式也生效）

## 位址匹配模式 (A)

| 值 | 模式 | 說明 |
|----|------|------|
| 0 | OFF | 禁用此區域 |
| 1 | TOR | Top of Range（區域 [pmpaddr_{i-1}, pmpaddr_i））|
| 2 | NA4 | 自然對齊 4 位元組 |
| 3 | NAPOT | 自然對齊 2^n 位元組（power of two）|

## xv8 的設定

```rust
// 允許所有實體記憶體（最大 2^54 - 1 = 16PB）
pmpaddr0::write(0x3fffffffffffff);

// 設定為 TOR + R + W
// 0xf = 0b1111 = TOR(1) + 1 + X(1) + R(1)
// 但 xv8 只設定 R + W，使用預設的 0x3fffffffffffff
pmpcfg0::write(0xf);
```

## TOR 模式

TOR (Top of Range) 表示範圍：
```
[pmpaddr_{i-1}, pmpaddr_i)
```

如果 pmpaddr0 = 0x1000，則：
- 區域 0：無效
- 區域 1：[0, 0x1000)

xv8 設定：
```rust
pmpaddr0::write(0x3fffffffffffff);
```

這形成：
```
[0, 0x3fffffffffffff]  // 所有記憶體
```

## 鎖定位 (L)

設定 L 位元後，即使 M 模式也受限於 PMP 規則。

xv8 **不設定** L 位元，讓 M 模式可以自由存取。

## 預設行為

如果沒有設定 PMP，S/U 模式預設可以存取所有記憶體。

xv8 明確設定 PMP 以確保安全邊界。

## 記憶體佈局對應

```
實體位址空間：
0x00000000 ────────────── 0x40000000  外設
0x40000000 ────────────── 0x88000000  PCIe MMIO
0x88000000 ────────────── 0x100000000 保留
0x100000000 ───────────── 0x8800000000 RAM (最大)

PMP 設定允許：
[0, 0x3fffffffffffff] = 0 到 16PB（涵蓋所有實體記憶體）
```

## 與 x86 MTR/PAT 的比較

| 特性 | RISC-V PMP | x86 MTRR |
|------|------------|----------|
| 數量 | 最多 16 個區域 | 最多 8 個 MTRR |
| 權限 | R/W/X | R/W/Cacheable |
| 鎖定 | 可選 | 作業系統設定 |
| 粒度 | 4B 起 | 4KB 起 |

## 用途

PMP 主要用於：
1. **隔離**：防止使用者程式存取核心記憶體
2. **信任區域**：保護安全敏感的記憶體區域
3. **除錯**：限制對特定記憶體範圍的存取

## 安全性含義

正確設定 PMP 可以防止：
- 使用者程式讀取核心資料
- 使用者程式執行核心程式碼
- 緩衝區溢位攻擊

## 範例：保護核心記憶體

```rust
// 限制 S 模式只能存取 RAM
pmpaddr0::write(RAM_END >> 2);  // RAM 結束位址
pmpcfg0::write(0x7);  // TOR + R + W (無 X)
```

## 錯誤處理

如果 S/U 模式嘗試存取 PMP 禁止的區域：
- 發生存取錯誤例外
- `scause` = Load/Store Access Fault
- 核心可以選擇終止程式或處理