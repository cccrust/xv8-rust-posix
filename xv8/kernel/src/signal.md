# 訊號系統 — signal.rs

訊號是 POSIX 程序的軟體中斷機制，用於通知程序發生了某個事件。

## 常見訊號

| 訊號 | 值 | 預設動作 | 說明 |
|------|-----|----------|------|
| SIGHUP | 1 | 終止 | 掛斷 |
| SIGINT | 2 | 終止 | 中斷 (Ctrl-C) |
| SIGKILL | 9 | 終止 | 強制殺死 |
| SIGSEGV | 11 | 終止+dump | 段錯誤 |
| SIGPIPE | 13 | 終止 | 寫入已關閉管道 |
| SIGALRM | 14 | 終止 | 定時器 |
| SIGTERM | 15 | 終止 | 終止請求 |

## 訊號狀態

```rust
pub struct SignalState {
    pub pending: AtomicUsize,     // 待處理訊號
    pub blocked: AtomicUsize,     // 阻塞的訊號
    pub in_handler: AtomicBool,   // 是否在處理常式中
}

impl SignalState {
    pub fn new() -> Self { ... }
    pub fn get_pending(&self) -> usize { ... }
    pub fn get_blocked(&self) -> usize { ... }
    pub fn set_pending(&self, sig: usize) { ... }
    pub fn clear_signal(&self, sig: usize) { ... }
}
```

## 訊號動作

```rust
pub struct SigAction {
    pub handler: usize,   // 處理常式位址 (0=預設, 1=忽略)
    pub flags: usize,      // SA_xxx 標誌
    pub mask: usize,      // 阻塞的額外訊號
}

const SIG_DFL: usize = 0;  // 預設動作
const SIG_IGN: usize = 1;  // 忽略
```

## 訊號框架

當訊號處理常式被呼叫時，使用者堆疊上有：

```rust
struct SigFrame {
    signo: i32,
    pad: 0,
    epc: u64,      // 返回位址
    ra: u64,       // 儲存的 ra
    sp: u64,       // 堆疊指標
    gp: u64,       // 全域指標
    tp: u64,       // 執行緒指標
    // ... 所有通用暫存器 ...
    oldmask: u64,  // 之前的 blocked mask
}
```

## 訊號傳遞

在 `trap.rs` 的 `deliver_pending_signals()` 中：

```rust
unsafe fn deliver_pending_signals(data: &mut ProcData) {
    loop {
        let pending = data.signals.get_pending();
        let blocked = data.signals.get_blocked();
        let unblocked = pending & !blocked;

        if unblocked == 0 {
            break;  // 沒有待處理訊號
        }

        let sig = unblocked.trailing_zeros() as usize + 1;
        data.signals.clear_signal(sig);

        match act.handler {
            // 預設動作
            0 => {
                if sig == SIGSTOP || sig == SIGCONT {
                    continue;
                }
                if is_fatal_signal(sig) {
                    proc::exit(-(sig as isize));
                }
            }
            // 忽略
            1 => {}
            // 自訂處理常式
            _ => {
                // 建立訊號框架
                let frame = SigFrame { ... };
                data.pagetable_mut().copy_to(frame_bytes, frame_va)?;

                // 修改 trapframe 跳到處理常式
                tf.epc = act.handler;
                tf.sp = frame_va;
                tf.a0 = sig;  // 第一個參數是訊號編號
                data.signals.in_handler.store(true);
                break;
            }
        }
    }
}
```

## sigaction 系統呼叫

```rust
pub fn sys_sigaction(args: &SyscallArgs) -> Result<(), SysError> {
    let sig = args.get_raw(0);
    let act_ptr = args.get_addr(1);
    let old_ptr = args.get_addr(2);

    let act: SigAction = read_from_user(act_ptr)?;

    // 儲存舊動作
    if old_ptr != 0 {
        write_to_user(old_ptr, data.sigactions[sig - 1])?;
    }

    // 設定新動作
    data.sigactions[sig - 1] = act;

    Ok(())
}
```

## sigprocmask

```rust
pub fn sys_sigprocmask(args: &SyscallArgs) -> Result<(), SysError> {
    let how = args.get_raw(0);
    let new_mask = args.get_raw(1);

    let old = data.signals.get_blocked();

    match how {
        SIG_BLOCK => data.signals.blocked.store(old | new_mask, ...),
        SIG_UNBLOCK => data.signals.blocked.store(old & !new_mask, ...),
        SIG_SETMASK => data.signals.blocked.store(new_mask, ...),
    }

    Ok(())
}
```

## sigreturn

用於從訊號處理常式返回：

```rust
pub fn sys_sigreturn(args: &SyscallArgs) -> Result<!, SysError> {
    let proc = current_proc();
    let data = unsafe { proc.data_mut() };

    // 恢復之前的 blocked mask
    data.signals.blocked.store(data.trapframe().oldmask as usize, ...);
    data.signals.in_handler.store(false);

    // trapframe 已被恢復，繼續執行
}
```

## 訊號發送

```rust
pub fn kill(pid: Pid) -> bool {
    for proc in PROC_TABLE.iter() {
        let mut inner = proc.inner.lock();
        if inner.pid == pid {
            // 設定待處理訊號
            proc.data().signals.set_pending(signal_number);

            // 如果程序在睡眠，喚醒它
            if inner.state == ProcState::Sleeping {
                inner.state = ProcState::Runnable;
            }
            return true;
        }
    }
    false
}
```

## 處理時機

訊號在返回使用者空間前檢查（`usertrapret()` 呼叫 `deliver_pending_signals`）。

## 限制

- xv8 不支援即時訊號（SIGRTMIN-SIGRTMAX）
- 訊號處理常式使用使用者堆疊（可自訂使用 sigaltstack）
- 不支援 SA_RESTART 標誌

## 相關主題

- [[trap]]：陷阱處理
- [[syscall]]：系統呼叫
- [[Process]]：程序管理