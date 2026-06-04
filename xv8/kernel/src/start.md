# Machine Mode 啟動 — start.rs

從 machine mode 過渡到 supervisor mode 的初始化程式碼。

## 執行流程

```
entry.S (machine mode)
    │
    ├── 設定堆疊
    ├── 呼叫 start()
    │
    ▼
start.rs::start() (machine mode)
    │
    ├── 設定 mstatus (MPP = Supervisor)
    ├── 設定 mepc = main
    ├── 停用虛擬記憶體 (satp = 0)
    ├── 委託中斷/例外到 supervisor
    ├── 設定 PMP
    ├── 設定計時器
    ├── 設定 tp = hartid
    │
    ├── asm!("mret")  ──────────────────┐
    │                                    │
    ▼                                    │
lib.rs::main() (supervisor mode) ◄────────┘
```

## 堆疊配置

```rust
#[repr(C, align(16))]
struct Stack([u8; 4096 * NCPU]);

static mut STACK0: Stack = Stack([0; 4096 * NCPU]);
```

每個 HART 有 4096 位元組的堆疊，16 位元組對齊。

## mstatus 設定

```rust
// 設定之前的特權模式為 supervisor
mstatus::set_mpp(mstatus::MPP_SUPERVISOR);
```

`mstatus.mpp` 決定 `mret` 返回後的特權模式。

## 中斷委託

```rust
// 委託所有中斷和例外到 supervisor
medeleg::write(0xffff);  // 委託例外
mideleg::write(0xffff); // 委託中斷

// 啟用 supervisor 的中斷
sie::write(sie::read() | sie::SEIE | sie::STIE | sie::SSIE);
```

## PMP 設定

```rust
// 讓 supervisor 可以訪問所有實體記憶體
// 0x3fffffffffffff = 全 1（47 位元）+ 讀寫執行
pmpaddr0::write(0x3fffffffffffff);
pmpcfg0::write(0xf);
```

## 計時器初始化

```rust
unsafe fn timer_init() {
    // 啟用 supervisor 計時器中斷
    mie::write(mie::read() | mie::STIE);

    // 啟用 sstc 擴展
    menvcfg::write(menvcfg::read() | (1 << 63));

    // 允許 supervisor 使用 stimecmp 和 time
    mcounteren::write(mcounteren::read() | 2);

    // 設定下次計時器中斷
    stimecmp::write(time::read() + 1_000_000);
}
```

## mret 指令

```rust
asm!("mret", options(noreturn));
```

`mret` 執行：
1. `mepc` → `pc`（跳轉到 `main`）
2. `mstatus.mpp` → 特權模式（變成 Supervisor）
3. `mstatus.mpie` → `mstatus.sie`（恢復中斷）

## HART ID

```rust
let id = mhartid::read();
tp::write(id);
```

`tp` 暫存器在 supervisor mode 儲存 HART ID。

## 多 HART 啟動

```
HART 0                      HART 1, 2, 3
│                           │
│ start()                    │
│   │                        │
│   ▼                        │
│ timer_init()             等待
│   │                        │
│   ▼                        │
│ main()                    start()
│   │                        │
│   │                        │
│   ▼                        │
│ scheduler() ───────────────┼───► scheduler()
│                           │
```

## 關鍵安全考量

1. **PMP 必須正確設定**：否則 supervisor 無法訪問記憶體
2. **計時器必須設定**：否則無法產生時鐘中斷進行程序切換
3. **中斷委託**：否則無法處理外部中斷

## 相關主題

- [[Boot]]：完整啟動流程
- [[memlayout]]：記憶體佈局
- [[riscv]]：RISC-V 特定功能
- [[proc]]：程序管理