# Boot — xv8 開機流程

xv8 從機器模式開機，最終進入監督者模式執行核心。

## 開機序列

```
┌─────────────────────────────────────────────┐
│ 硬體重置                                     │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 0x1000: Boot ROM (QEMU 提供)                │
│ 跳轉到 0x80000000                           │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ 0x80000000: kernel.entry (_entry)           │
│ - 設定堆疊 (每個 HART 獨立 4KB 堆疊)         │
│ - 呼叫 start()                             │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ start() — start.rs                           │
│ - 設定 mstatus (MPP = S-mode)               │
│ - 設定 mepc = main                          │
│ - 設定 PMP (允許 S 模式存取所有記憶體)       │
│ - delegate traps to S-mode                  │
│ - 初始化計時器                               │
│ - mret 進入 S-mode main()                   │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ main() — kernel/main.rs                      │
│ - 初始化記憶體配置器                         │
│ - 初始化 virtio磁碟                         │
│ - 初始化程序系統                            │
│ - 建立第一個 shell (init)                   │
└─────────────────────────────────────────────┘
```

## 堆疊設定

每個 HART 有獨立 4KB 堆疊：

```rust
asm!(
    "la sp, STACK0",      // 載入 STACK0 位址
    "li a0, 4096",        // 4096 bytes per stack
    "csrr a1, mhartid",   // 讀取 HART ID
    "addi a1, a1, 1",     // HART 1 的偏移是 1*4096
    "mul a0, a0, a1",     // 計算偏移
    "add sp, sp, a0",     // 設定堆疊指標（向下生長）
);
```

堆疊佈局：
```
STACK0                    ← 最高位址
STACK0 + 4096            ← HART 0 堆疊頂端
STACK0 + 2*4096          ← HART 1 堆疊頂端
...
```

## PMP — 實體記憶體保護

設定 S 模式可存取的記憶體範圍：

```rust
// 允許 0x3fffffffffffff 以下的所有記憶體
pmpaddr0::write(0x3fffffffffffff);
pmpcfg0::write(0xf);  // TOR + R + W
```

PMP 配置格式：
```
0xf = 0b1111
     │││└─ A[1] = TOR
     ││└─ A[0] = 1
     │└─ X = 1
     └─ R = 1
```

## 特權模式切換

```rust
// 設定返回到 S 模式
mstatus::set_mpp(mstatus::MPP_SUPERVISOR);

// 設定返回位址為 main
mepc::write(main as *const () as usize);

// 執行 mret 後進入 S 模式 main()
asm!("mret", options(noreturn));
```

## 計時器初始化

```rust
unsafe fn timer_init() {
    // 啟用 S 模式計時器中斷
    mie::write(mie::read() | mie::STIE);

    // 啟用 sstc 擴充（stimecmp）
    menvcfg::write(menvcfg::read() | (1 << 63));

    // 允許 S 模式使用 time 和 stimecmp
    mcounteren::write(mcounteren::read() | 2);

    // 設定第一個計時器中斷（~1ms 後）
    stimecmp::write(time::read() + 1000000);
}
```

## Trap 委派

所有例外和中斷委派給 S 模式處理：

```rust
medeleg::write(0xffff);  // 例外委派
mideleg::write(0xffff);  // 中斷委派

// 啟用所有 S 模式中斷
sie::write(sie::read() | sie::SEIE | sie::STIE | sie::SSIE);
```

## 多 HART 支援

每個 HART 執行 `start()` 然後進入 `main()`。

```rust
let id = mhartid::read();
tp::write(id);  // tp 暫存器保存 HART ID
```

`tp` 暫存器用於區分不同 HART 的資料結構。

## 與 xv6 的比較

| 步驟 | xv6 (C) | xv8 (Rust) |
|------|---------|------------|
| entry | 組語 | 組語 + Rust |
| 堆疊設定 | 組語 | 組語內嵌 |
| CSR 設定 | C 巨集 | Rust 函式 |
| 計時器 | 組語 | Rust + 組語 |

## 安全性考量

1. **PMP**：防止 S 模式存取不當記憶體
2. **medeleg/mideleg**：隔離 M 模式處理的 trap
3. **mret**：只能從較高特權模式返回

## 故障排除

常見開機問題：
- 堆疊未對齊 → 堆疊溢位
- PMP 設定過嚴 → 存取錯誤
- trap 委派不完整 → 未知 trap 類型
- 計時器未初始化 → 無法排程