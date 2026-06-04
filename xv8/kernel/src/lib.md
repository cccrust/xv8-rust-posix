# 核心初始化 — lib.rs

xv8 核心的進入點和初始化流程。

## 進入點架構

```
QEMU 載入
    │
    ▼
entry.S (entry.asm)
    │ 設定堆疊指標
    │ 呼叫 start
    ▼
start.rs (machine mode)
    │ 設定時鐘
    │ 進入 supervisor mode
    ▼
lib.rs::main()
    │ 各子系統初始化
    ▼
scheduler() 永遠循環
```

## 核心模組組織

```rust
#![no_std]
extern crate alloc;

pub(crate) mod printf;
pub(crate) mod error;
pub mod buf;          // 緩衝區塊快取
pub mod console;      // 主控台
pub mod e1000;       // 網卡驅動
pub mod entry;
pub mod exec;         // ELF 執行
pub mod file;         // 檔案抽象
pub mod fs;           // 檔案系統
pub mod kalloc;       // 記憶體配置
pub mod kernelvec;    // 核心陷阱向量
pub mod log;          // 日誌
pub mod memlayout;    // 記憶體佈局
pub mod net;          // 網路堆疊
pub mod param;         // 參數
pub mod pci;           // PCI 匯流排
pub mod pipe;          // 管道
pub mod plic;          // 中斷控制器
pub mod proc;          // 程序管理
pub mod riscv;         // RISC-V 特定功能
pub mod rng;           // 隨機數
pub mod sleeplock;     // 睡眠鎖
pub mod signal;        // 信號
pub mod spinlock;      // 自旋鎖
pub mod start;
pub mod swtch;         // 上下文切換
pub mod sync;          // 同步原語
pub mod syscall;       // 系統呼叫
pub mod sysfile;       // 檔案系統呼叫
pub mod sysnet;        // 網路呼叫
pub mod sysproc;       // 程序呼叫
pub mod trampoline;
pub mod trap;          // 陷阱處理
pub mod uart;          // 序列埠
pub mod virtio_disk;   // 磁碟驅動
pub mod vm;            // 虛擬記憶體
```

## 主要初始化流程

```rust
pub fn main() -> ! {
    let cpu_id = unsafe { proc::current_id() };

    if cpu_id == 0 {
        // 只有第一個 HART 執行初始化
        console::init();
        println!("xv8 kernel is booting");

        // 記憶體
        kalloc::init();
        rng::init();
        vm::init();
        vm::init_hart();

        // 程序和陷阱
        proc::init();
        trap::init();
        trap::init_hart();

        // 中斷控制器
        plic::init();
        plic::init_hart();

        // 第一個使用者程序
        proc::user_init();

        // 檔案系統和裝置
        buf::init();
        virtio_disk::init();
        net::init();
        pci::init();
        e1000::init();

        println!("hart {} is starting", cpu_id);
        STARTED.store(true, Ordering::SeqCst);
    } else {
        // 其他 HART 等待初始化完成
        while !STARTED.load(Ordering::SeqCst) {
            core::hint::spin_loop()
        }

        println!("hart {} is starting", cpu_id);

        unsafe {
            vm::init_hart();
            trap::init_hart();
            plic::init_hart();
        }
    }

    // 開始排程（永不返回）
    unsafe { proc::scheduler() };
}
```

## 初始化順序

```
1. console::init()
   ↓
2. kalloc::init()        - Buddy 配置器
   ↓
3. rng::init()           - 隨機數產生器
   ↓
4. vm::init()            - 核心頁表
   ↓
5. vm::init_hart()       - 啟用分頁
   ↓
6. proc::init()          - 程序表
   ↓
7. trap::init()          - 陷阱處理
   ↓
8. plic::init()          - 中斷控制器
   ↓
9. proc::user_init()     - 第一個程序
   ↓
10. buf::init()          - 區塊緩衝
   ↓
11. virtio_disk::init()  - 磁碟驅動
   ↓
12. net::init()          - 網路堆疊
   ↓
13. pci::init()          - PCI 匯流排
   ↓
14. e1000::init()        - 網卡驅動
   ↓
15. proc::scheduler()    - 開始排程
```

## 多 HART 同步

```rust
static STARTED: AtomicBool = AtomicBool::new(false);

// 主 HART 設定為 true
STARTED.store(true, Ordering::SeqCst);

// 其他 HART 等待
while !STARTED.load(Ordering::SeqCst) {
    core::hint::spin_loop()
}
```

## 恐慌處理

```rust
pub fn panic_handler(info: &core::panic::PanicInfo<'_>) -> ! {
    printf::panic(info)
}
```

## 各子系統初始化職責

| 子系統 | 職責 |
|--------|------|
| kalloc | 配置Buddy配置器的記憶體範圍 |
| vm | 建立核心頁表映射 |
| proc | 配置程序表和核心堆疊 |
| trap | 設定陷阱向量 |
| plic | 設定中斷優先級 |
| buf | 初始化 LRU 鏈表 |
| fs | 讀取超級區塊，初始化日誌 |
| net | 啟動網路執行緒 |
| pci | 枚舉 PCI 裝置 |

## 相關主題

- [[Boot]]：完整啟動流程
- [[Process]]：程序管理
- [[Trap]]：陷阱處理
- [[vm]]：虛擬記憶體