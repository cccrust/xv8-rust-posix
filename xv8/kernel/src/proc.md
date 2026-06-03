# 程序管理 — proc.rs

程序管理是作業系統的核心功能之一，負責行程的建立、調度、記憶體管理。

## 程序結構

```rust
struct Proc {
    id: usize,              // 程序表索引
    inner: SpinLock<ProcInner>,  // 保護狀態
    data: UnsafeCell<ProcData>,   // 私有資料
}

struct ProcInner {
    state: ProcState,       // 行程狀態
    channel: Option<Channel>, // 睡眠通道
    killed: bool,           // 是否被殺死
    xstate: isize,          // 退出狀態
    pid: Pid,               // 程序 ID
}

struct ProcData {
    kstack: VA,             // 核心堆疊
    pagetable: Option<Uvm>, // 使用者頁表
    trapframe: Option<Box<TrapFrame>>,
    context: Context,       // 上下文切換
    open_files: [Option<File>; 64],
    cwd: Inode,             // 目前目錄
    name: String,
    // ... 更多欄位
}
```

## 程序狀態機

```
                 fork()
                    ↓
┌──────────────────────────────────────┐
│              Used                     │
└──────────────────────────────────────┘
                    ↓
               setup_user()
                    ↓
┌──────────────────────────────────────┐
│            Runnable                   │◄──────────┐
└──────────────────────────────────────┘            │
                    ↓                               │
              scheduler()                          │ yield()
                    ↓                               │
┌──────────────────────────────────────┐            │
│             Running                   │────────────┘
└──────────────────────────────────────┘
                    ↓
                 exit()
                    ↓
┌──────────────────────────────────────┐
│             Zombie                    │
└──────────────────────────────────────┘
                    ↓
                 wait()
                    ↓
┌──────────────────────────────────────┐
│             Unused                    │
└──────────────────────────────────────┘

                 sleep()
                    ↓
┌──────────────────────────────────────┐
│            Sleeping                   │
└──────────────────────────────────────┘
                    ↓
               wakeup()
                    ↓
                 (Runnable)
```

## 行程表

```rust
const NPROC: usize = 64;

pub static PROC_TABLE: ProcTable = ProcTable::new();

struct ProcTable {
    table: [UnsafeCell<Proc>; NPROC],
    parents: SpinLock<[Option<usize>; NPROC]>,  // parent[child_id] = Some(parent_id)
}
```

## 程序建立 (fork)

```rust
pub fn fork() -> Result<Pid, KernelError> {
    // 1. 配置新的程序結構
    let (new_proc, new_inner) = PROC_TABLE.alloc()?;
    new_inner = new_proc.setup_user(new_inner)?;

    let new_data = unsafe { new_proc.data_mut() };

    // 2. 複製使用者記憶體（COW）
    let new_pagetable = new_data.pagetable_mut();
    data.pagetable_mut().copy(new_pagetable, size)?;

    // 3. 複製暫存器框架
    new_trapframe.clone_from(trapframe);
    new_trapframe.a0 = 0;  // 子行程返回 0

    // 4. 複製開啟的檔案
    for (i, file) in data.open_files.iter_mut().enumerate() {
        new_data.open_files[i] = Some(file.dup());
    }

    // 5. 設定為可執行
    new_inner.state = ProcState::Runnable;

    Ok(pid)
}
```

## Copy-on-Write fork

```rust
pub fn copy(&mut self, child: &mut Uvm, size: usize) -> Result<(), VmError> {
    for i in (0..size).step_by(PGSIZE) {
        let pte = self.walk_mut(VA::from(i), false)?;

        if pte.is_w() {
            // 清除寫權限，設定 COW 位
            *pte &= !PTE_W;
            *pte |= PTE_COW;
        }

        // 映射到同一個實體頁
        child.map_pages(VA::from(i), pte.as_pa(), PGSIZE, pte.flags())?;

        // 增加參考計數
        kalloc::increment_ref(pte.as_pa());
    }
    Ok(())
}
```

## 上下文切換

```rust
struct Context {
    ra: usize,  // 返回位址
    sp: usize,   // 堆疊指標
    // callee-saved 暫存器
    s0: usize, s1: usize, ..., s11: usize,
}

// scheduler 中的切換
unsafe { swtch(&mut cpu.context, &proc.data().context) };
```

## 排程器

```rust
pub unsafe fn scheduler() -> ! {
    loop {
        interrupts::enable();
        interrupts::disable();

        for proc in PROC_TABLE.iter() {
            let mut inner = proc.inner.lock();

            if inner.state == ProcState::Runnable {
                inner.state = ProcState::Running;
                cpu.proc.replace(proc);

                // 切換到程序
                unsafe { swtch(&mut cpu.context, &proc.data().context) };

                cpu.proc.take();
            }
        }

        // 沒有可執行的地行程，進入待機
        unsafe { asm!("wfi") };
    }
}
```

## Sleep / Wakeup

```rust
pub fn sleep<T>(channel: Channel, condition_lock: SpinLockGuard<'_, T>)
    -> SpinLockGuard<'_, T> {
    let proc = current_proc();
    let mut inner = proc.inner.lock();

    // 釋放條件鎖，進入睡眠
    let condition_mutex = SpinLock::unlock(condition_lock);

    inner.channel = Some(channel);
    inner.state = ProcState::Sleeping;

    // 切換到排程器
    inner = sched(inner, context);

    inner.channel = None;

    // 重新取得條件鎖
    condition_mutex.lock()
}

pub fn wakeup(channel: Channel) {
    for proc in PROC_TABLE.iter() {
        let mut inner = proc.inner.lock();
        if inner.state == ProcState::Sleeping && inner.channel == Some(channel) {
            inner.state = ProcState::Runnable;
        }
    }
}
```

## 僵尸程序回收

```rust
pub fn wait(addr: VA) -> Option<Pid> {
    let current_id = current_proc().id;

    loop {
        for proc in PROC_TABLE.iter() {
            if parents[proc.id] == Some(current_id) {
                let inner = proc.inner.lock();

                if inner.state == ProcState::Zombie {
                    // 取得退出狀態
                    if addr != 0 {
                        copy_to_user(&inner.xstate.to_le_bytes(), addr);
                    }

                    // 清除父子關係
                    parents[proc.id] = None;

                    // 釋放程序資源
                    proc.free(inner);

                    return Some(pid);
                }
            }
        }

        // 沒有找到僵尸子程序，進入睡眠
        if !have_kids || current_proc.inner.lock().killed {
            return None;
        }

        sleep(Channel::Proc(current_id), parents);
    }
}
```

## 核心執行緒

```rust
pub fn spawn_kernel_thread<F>(f: F, name: &str)
where
    F: FnOnce() + Send + 'static,
{
    let (proc, inner) = PROC_TABLE.alloc().unwrap();
    let mut inner = proc.setup_kernel(inner, Box::new(f));

    let data = unsafe { proc.data_mut() };
    data.name.push_str(name);
    data.cwd = Path::new("/").resolve().unwrap();

    inner.state = ProcState::Runnable;
}
```

## 懶惰配置 (Lazy Allocation)

```rust
pub unsafe fn grow(n: isize, lazy: bool) -> Result<usize, KernelError> {
    if n > 0 {
        if lazy {
            // 只增加 size，不立即配置記憶體
            size += n as usize;
        } else {
            // 立即配置
            size = data.pagetable_mut().alloc(size, size + n as usize, PTE_W)?;
        }
    }
    // ...
}
```

## 相關主題

- [[Sv39]]：分頁與記憶體管理
- [[Trap]]：陷阱處理與上下文切換
- [[Syscall]]：系統呼叫
- [[exec]]：程式執行