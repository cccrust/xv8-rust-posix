# Scheduler（排程器）

排程器是 xv8 核心中決定下一個運行哪個程序的元件。xv8 使用簡單的輪詢（Round-Robin）排程演算法。

## 排程觸發時機

排程可能在以下情況被觸發：

1. **程序主動放棄 CPU**：呼叫 `yield`、`sleep`、`wait`、`exit`
2. **時間配額耗盡**：計時器中斷發生
3. **等待的資源就緒**：被阻塞的程序被 `wakeup` 喚醒

## 排程器初始化

在核心啟動時，`proc::init()` 分配程序表記憶體並建立 init 程序（pid=1）：

```rust
pub fn init() {
    // 分配程序表（64 個槽位）
    for i in 0..NPROC {
        proc_table[i].state = ProcState::UNUSED;
    }
    // 建立 init 程序
    user_init();
}
```

## 程序表

xv8 支援最多 64 個並發程序，儲存在靜態陣列中：

```rust
const NPROC: usize = 64;
static mut PROCS: [Proc; NPROC] = [Proc::zeroed(); NPROC];
```

每個程序槽位可以是 UNUSED（空閒）或已分配給某程序。

## 輪詢排程

xv8 的排程器簡單遍歷程序表，找到第一個 RUNNABLE 程序：

```rust
pub fn scheduler() -> ! {
    loop {
        let mut p: *mut Proc = core::ptr::null_mut();
        // 尋找可運行的程序
        for i in 0..NPROC {
            let proc = &mut PROCS[i];
            if proc.state == ProcState::RUNNABLE {
                p = proc;
                break;
            }
        }
        // 如果找到，執行上下文切換
        if !p.is_null() {
            unsafe { run(p) };
        }
    }
}
```

`run()` 函式執行實際的上下文切換：

1. 將目前程序標記為 RUNNING
2. 呼叫 `swtch` 切換到新程序的上下文
3. 新程序從上次離開的地方恢復執行

## 上下文切換

上下文切換是排程的核心，由 `swtch.rs` 實現：

```rust
pub fn swtch(from: &mut Context, to: &Context) {
    unsafe {
        core::arch::asm!(
            // 保存呼叫者保存暫存器
            "sd ra, 0(a0)",
            "sd sp, 8(a0)",
            "sd s0, 16(a0)",
            "sd s1, 24(a0)",
            // ...
            // 恢復被切換到的上下文
            "ld ra, 0(a1)",
            "ld sp, 8(a1)",
            "ld s0, 16(a1)",
            "ld s1, 24(a1)",
            // ...
            "ret"
        );
    }
}
```

`swtch` 只保存和恢復一小部分暫存器（s0-s11 是被呼叫者保存，C caller's saved registers）。其餘暫存器（ra、sp、a0-a7 等）在函式呼叫期間已經保存在堆疊上。

## 上下文結構

```rust
#[repr(C)]
pub struct Context {
    ra: usize,    // 返回位址
    sp: usize,     // 堆疊指標
    s0: usize,    // s0-s11 是被呼叫者保存暫存器
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}
```

## 睡眠與喚醒

當程序需要等待 I/O 或其他資源時：

```rust
pub fn sleep(channel: usize, lock: &SpinLock) {
    let p = current();
    p.lock.with_lock(|state| {
        *state = ProcState::SLEEPING;
    });
    drop(lock);
    unsafe { sched() };
    // 喚醒後從這裡繼續
}
```

`sleep` 將程序狀態設為 SLEEPING，釋放鎖，然後呼叫 `sched()` 進行排程。`sched()` 呼叫 `swtch` 切換到排程器。

當資源就緒時，`wakeup` 找到等待該資源的程序並將其標記為 RUNNABLE：

```rust
pub fn wakeup(channel: usize) {
    for i in 0..NPROC {
        let p = &mut PROCS[i];
        if p.is_sleeping(channel) {
            p.set_runnable();
        }
    }
}
```

## 時間配額

xv8 不使用嚴格的時間片（time slice）搶佔。每個程序可以運行任意長時間，直到：

1. 呼叫 `yield` 主動放棄
2. 呼叫阻塞系統呼叫（read、write、wait 等）
3. 發生計時器中斷

計時器中斷目前在 xv8 中主要用於維持系統時間，而非搶佔。

## 閒置程序

當沒有任何 RUNNABLE 程序時，排程器進入閒置循環：

```rust
loop {
    asm!("wfi");  // 等待中斷
}
```

`wfi`（Wait for Interrupt）指令讓 CPU 進入低功耗狀態直到中斷發生，節省電力。

## 核心堆疊

每個程序有自己的核心堆疊（4KB），用於在核心態執行系統呼叫時：

```rust
unsafe {
    let kstack = kalloc::alloc();
    p.kstack = kstack;
}
```

當發生 trap 時，硬體會自動使用即將運行的程序的核心堆疊。`kernelvec` 在這個堆疊上保存使用者的暫存器。

## 安全考量

排程器需要確保：
- 不可中斷（interrupt disabled）時不執行太久
- 自旋鎖不能在 interrupt enabled 的情況下持有太久
- 程序切換時不丟失任何狀態

xv8 使用 `intr_off()` 和 `intr_on()` 來控制中斷狀態。

## 相關主題

- [[Process]]：程序狀態與管理
- [[Trap]]：上下文切換如何與 trap 整合
- [[RISC-V]]：`swtch` 使用的組語細節