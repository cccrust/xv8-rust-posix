# HART — 硬體執行緒

RISC-V HART (HardARE Thread) 是可以獨立執行指令的硬體單元。

## 概念

HART = 實體核心 + 獨立的 PC 和暫存器集合

一個 HART 類似於 x86 的邏輯核心或 ARM 的 PE (Processing Element)。

## xv8 的 HART 支援

```rust
pub const NCPU: usize = 8;  // 最大 HART 數
```

QEMU 預設使用 4 個 HART：
```
qemu-system-riscv64 -smp 4
```

## 啟動序列

每個 HART 執行 `_entry()`：

```rust
pub unsafe fn _entry() -> ! {
    asm!(
        "la sp, STACK0",     // 載入堆疊基底
        "li a0, 4096",       // 堆疊大小
        "csrr a1, mhartid",  // 讀取 HART ID
        "addi a1, a1, 1",
        "mul a0, a0, a1",
        "add sp, sp, a0",    // 設定堆疊
    );
    start()
}
```

## 堆疊佈局

```rust
#[repr(C, align(16))]
struct Stack([u8; 4096 * NCPU]);

static mut STACK0: Stack = Stack([0; 4096 * NCPU]);
```

每個 HART 有 4096 位元組堆疊：
```
HART 0: STACK0[0..4096]
HART 1: STACK0[4096..8192]
HART 2: STACK0[8192..12288]
...
```

## HART ID

```rust
let id = mhartid::read();
tp::write(id);  // 保存到 tp 暫存器
```

`tp` 暫存器保存 HART ID，讓每個核心知道自己的身份。

## 多 HART 初始化

```rust
// 每個 HART 都執行
pub unsafe fn start() -> ! {
    // ...

    let id = mhartid::read();
    tp::write(id);  // 儲存 HART ID

    asm!("mret", options(noreturn));
}
```

## HART 之間的差異

每個 HART 有：
- 獨立的 PC
- 獨立的通用暫存器
- 獨立的 CSR（但某些共享）
- 獨立的暫時計數器
- 共享的物理記憶體

## 跨 HART 通訊

### 軟體中斷

```rust
// 從 HART 0 發送到 HART 1
// 寫入 clint 的 MSIP
*(CLINT + 0x4 + hart_id * 4) = 1;
```

### 共享記憶體

由於共享記憶體，HART 可以通過共享變數通訊：

```rust
static mut SHARED_DATA: AtomicUsize = AtomicUsize::new(0);

unsafe {
    SHARED_DATA.store(value, Ordering::SeqCst);
}
```

## 同步

多 HART 需要同步機制：

### 中斷停用

```rust
disable();  // 停用中斷
// 臨界區
enable();
```

### 自旋鎖

```rust
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub fn lock(&self) {
        while self.locked.compare_exchange(
            false, true, Ordering::Acquire, Ordering::Relaxed
        ).is_err() {
            // busy wait
        }
    }
}
```

## NUMA

在真實硬體上，HART 可能有不同的記憶體親和度。

xv8 目前不考慮 NUMA，所有記憶體對所有 HART 平等。

## 與作業系統的關係

作業系統看到每個 HART 為一個 CPU。

```rust
// 取得目前 HART 數量
let ncores = NCPU;
```

## QEMU 模擬

QEMU 模擬多 HART：
```
-info cpus  // 查看 HART 狀態
-smp 4      // 4 個 HART
```

## 排程

xv8 的排程器將程序分配到可用 HART：

```rust
struct Proc {
    hartid: usize,  // 目前執行的 HART
    // ...
}
```

## 計時器

每個 HART 有獨立的計時器：

```rust
stimecmp::write(next_interrupt_time);
// 只影響當前 HART
```

## 快取一致性

RISC-V 不保證快取一致性（取決於實作）。

在 xv8 中，我們假設：
- 使用快取失效指令（fence）
- 或使用 non-cacheable 記憶體（MMIO）

```rust
vma::sfence();  // 刷新 TLB/快取
```

## 斷言

多 HART 的常見錯誤：
1. **死結**：兩個 HART 互相等待對方持有的鎖
2. **競爭條件**：未同步的共享資料存取
3. **優先權反轉**：低優先權程序持有鎖，高優先權無法執行

## 效能

- 增加 HART 數量通常提升吞吐量
- 但有鎖競爭和同步開銷
- 最佳 HART 數取決於工作負載

## 顯示 QEMU 中的 HART

```bash
# 在 QEMU 內
cat /proc/cpuinfo  # 如果有作業系統
# 或
info cpus         # QEMU monitor
```