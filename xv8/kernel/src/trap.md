# 陷阱處理 — trap.rs

陷阱（Trap）是 RISC-V 控制轉移到核心的機制，包括例外、系統呼叫和中斷。

## 陷阱類型

| 類型 | 觸發條件 | 目的地 |
|------|----------|--------|
| 例外 | 指令執行錯誤 | usertrap / kerneltrap |
| 系統呼叫 | ecall 指令 | usertrap |
| 中斷 | 外部訊號 | usertrap / kerneltrap |

## RISC-V 陷阱機制

```
┌─────────────────────────────────────────────────────┐
│                   使用者模式                         │
│  uservec (trampoline.S)                              │
│     ↓ 儲存使用者暫存器                               │
│  usertrap() ─────────────────────────────────────────┤
├─────────────────────────────────────────────────────┤
│                   核心模式                           │
│  kernelvec (kernelvec.S)                             │
│     ↓ 儲存核心暫存器                                 │
│  kerneltrap()                                        │
└─────────────────────────────────────────────────────┘
```

## 陷阱向量設定

```rust
// 在使用者模式觸發時，stvec 指向 uservec
stvec::write(uservec as usize);

// 在核心模式觸發時，stvec 指向 kernelvec
stvec::write(kernelvec as usize);
```

## TrapFrame

儲存使用者暫存器，用於在陷阱處理時保存上下文：

```rust
struct TrapFrame {
    kernel_satp: usize,   // 核心頁表
    kernel_sp: usize,     // 核心堆疊頂
    kernel_trap: usize,   // usertrap 位址
    epc: usize,           // 例外程式計數器
    kernel_hartid: usize, // HART ID
    // 通用暫存器
    ra: usize, sp: usize, gp: usize, tp: usize,
    t0: usize, t1: usize, t2: usize,
    a0: usize, a1: usize, ..., a7: usize,
    s0: usize, s1: usize, ..., s11: usize,
    t3: usize, t4: usize, t5: usize, t6: usize,
}
```

## usertrap() 處理流程

```rust
pub unsafe fn usertrap() {
    // 確保來自使用者模式
    assert!((sstatus::read() & sstatus::SPP) == 0);

    // 切換到核心陷阱向量
    stvec::write(kernelvec as usize);

    let (proc, data) = proc::current_proc_and_data_mut();
    let (pagetable, trapframe) = data.pagetable_and_trapframe_mut();

    // 儲存使用者 PC
    trapframe.epc = sepc::read();

    let scause = scause::Scause::from(scause::read());

    match scause.cause() {
        // 系統呼叫
        scause::Trap::Exception(scause::Exception::EnvironmentCall) => {
            trapframe.epc += 4;  // 跳过 ecall 指令
            interrupts::enable();
            syscall(trapframe);  // 分派系統呼叫
        }

        // 頁面錯誤（懶惰配置）
        scause::Trap::Exception(scause::Exception::StorePageFault) |
        scause::Trap::Exception(scause::Exception::LoadPageFault) => {
            if log!(pagetable.vmfault(VA::from(stval::read()))).is_err() {
                proc.inner.lock().killed = true;  // 無效記憶體，殺死程序
            }
        }

        // 裝置中斷
        scause::Trap::Interrupt(intr) if device_interrupt(intr).is_some() => {
            // 處理 UART、VirtIO、E1000 中斷
        }

        // 其他（錯誤）
        _ => {
            println!("unexpected trap");
            proc.inner.lock().killed = true;
        }
    }

    if proc.inner.lock().killed {
        proc::exit(-1);
    }

    if which_dev == Some(InterruptType::Timer) {
        proc::r#yield();  // 時間片用完，調度其他程序
    }

    usertrapret();  // 返回使用者空間
}
```

## 中斷類型

```rust
enum InterruptType {
    Device,  // 外部裝置中斷
    Timer,   // 計時器中斷
}

fn device_interrupt(intr: scause::Interrupt) -> Option<InterruptType> {
    match intr {
        // 外部中斷 (PLIC)
        scause::Interrupt::SupervisorExternal => {
            let irq = plic::claim();
            match irq {
                UART0_IRQ => uart::handle_interrupt(),
                VIRTIO0_IRQ => virtio_disk::handle_interrupt(),
                E1000_IRQ => e1000::handle_interrupt(),
                _ => {}
            }
            plic::complete(irq);
            Some(InterruptType::Device)
        }

        // 計時器中斷
        scause::Interrupt::SupervisorTimer => {
            clock_intr();
            Some(InterruptType::Timer)
        }

        _ => None,
    }
}
```

## 返回使用者空間

```rust
pub unsafe fn usertrapret() {
    // 傳遞信號處理
    deliver_pending_signals(data);

    interrupts::disable();

    // 設定 trampoline 位址
    let trampoline_uservec = TRAMPOLINE + (uservec - trampoline);
    stvec::write(trampoline_uservec);

    // 填寫 trapframe 的核心資訊
    trapframe.kernel_satp = satp::read();  // 核心頁表
    trapframe.kernel_sp = (kstack + NKSTACK_PAGES * PGSIZE);  // 核心堆疊
    trapframe.kernel_trap = usertrap as usize;
    trapframe.kernel_hartid = tp::read();

    // 設定 SPP 為使用者模式
    let mut x = sstatus::read();
    x &= !sstatus::SPP;  // 清除
    x |= sstatus::SPIE;  // 啟用中斷
    sstatus::write(x);

    // 設定返回 PC
    sepc::write(trapframe.epc);

    // 計算 userret 位址
    let trampoline_userret = TRAMPOLINE + (userret - trampoline);

    // 切換到使用者頁表，恢復暫存器
    trampoline_userret(user_satp);
}
```

## 時鐘中斷

```rust
pub fn clock_intr() {
    let hart = proc::current_id();

    if hart == 0 {
        let mut ticks = TICKS.lock();
        *ticks += 1;
        proc::wakeup(Channel::Ticks);
    }

    // 設定下次計時器中斷
    unsafe { stimecmp::write(time::read() + 1_000_000) };
}
```

## 懶惰配置與 COW

vmfault 處理三種情況：

```rust
pub fn vmfault(&mut self, va: VA) -> Result<PA, VmError> {
    // 情況 1：超出邊界
    if va >= data.size {
        err!(VmError::InvalidAddress);
    }

    // 情況 2：COW 頁面，需要複製
    if pte.is_cow() {
        let old_pa = pte.as_pa();

        // 配置新頁面
        let mem = Box::<Page>::try_new_zeroed()?;
        let new_pa = PA::from(Box::into_raw(mem) as usize);

        // 複製內容
        ptr::copy_nonoverlapping(
            old_pa.as_mut_ptr(),
            new_pa.as_mut_ptr(),
            PGSIZE,
        );

        // 安裝新頁面，啟用寫權限
        *pte = new_pa.as_pte() | PTE_W | PTE_R | PTE_U;

        // 減少舊頁面參考計數
        drop(Box::from_raw(old_pa.as_mut_ptr()));

        return Ok(new_pa);
    }

    // 情況 3：懶惰配置的頁面，配置並映射
    let mem = Box::<Page>::try_new_zeroed()?;
    self.map_pages(va, PA::from(Box::into_raw(mem) as usize), PGSIZE, PTE_W | PTE_U | PTE_R)?;
    Ok(pa)
}
```

## 信號傳遞

在返回使用者空間前檢查待處理信號：

```rust
fn deliver_pending_signals(data: &mut ProcData) {
    loop {
        let pending = data.signals.get_pending();
        let blocked = data.signals.get_blocked();
        let unblocked = pending & !blocked;

        if unblocked == 0 {
            break;
        }

        let sig = unblocked.trailing_zeros() as usize + 1;
        data.signals.clear_signal(sig);

        match act.handler {
            // 預設處理
            0 => {
                if is_fatal_signal(sig) {
                    proc::exit(-(sig as isize));
                }
            }
            // 忽略
            1 => {}
            // 自訂處理常式
            _ => {
                // 建立信號框架
                let frame = signal::SigFrame { ... };

                // 修改 trapframe 跳到處理常式
                tf.epc = act.handler;
                tf.sp = frame_va;
                tf.a0 = sig;
            }
        }
    }
}
```

## trampoline.S 的角色

trampoline 頁面同時映射到：
- 使用者頁表的最高位址（不可訪問）
- 核心頁表的最高位址

這允許無縫過渡：
```
uservec:  保存使用者暫存器到 trapframe
           ↓
          切換到核心頁表
           ↓
usertrap: 處理陷阱
           ↓
usertrapret: 恢復 trapframe 中的核心資訊
           ↓
userret:  恢復使用者暫存器，sret 返回
```

## 相關主題

- [[Sv39]]：分頁機制
- [[Process]]：程序管理
- [[Syscall]]：系統呼叫
- [[Interrupts]]：中斷控制器