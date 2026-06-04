# 自旋鎖 — spinlock.rs

自旋鎖是最基本的互斥同步原語，適用於短期鎖。

## 特性

- 停用中斷後取得鎖
- 忙等待（spin）直到取得鎖
- 確保不會發生於中斷處理常式中

## 資料結構

```rust
pub struct SpinLock<T> {
    name: &'static str,
    cpu: AtomicPtr<Cpu>,     // 持有者的 CPU 指標
    data: UnsafeCell<T>,      // 受保護的資料
}

pub struct SpinLockGuard<'a, T: 'a> {
    lock: &'a SpinLock<T>,
    _intr_lock: InterruptLock,  // 釋放時復原中斷狀態
}
```

## 取得鎖

```rust
pub fn lock(&self) -> SpinLockGuard<'_, T> {
    // 1. 停用中斷並取得 CPU 鎖
    let intr_lock = proc::lock_current_cpu();

    // 2. 確保尚未持有此鎖
    unsafe {
        assert!(!self.holding(), "acquire spinlock {}", self.name);
    }

    // 3. 忙等待直到成功交換
    loop {
        if self.cpu.compare_exchange(
            ptr::null_mut(),
            unsafe { proc::current_cpu() },  // 取得目前 CPU
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok() {
            break SpinLockGuard {
                lock: self,
                _intr_lock: intr_lock,
            };
        }
        hint::spin_loop()
    }
}
```

## 釋放鎖

```rust
impl Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        // 確認仍然持有鎖
        assert!(unsafe { self.lock.holding() }, "release lock {}", self.lock.name);

        // 清除持有者指標
        self.lock.cpu.store(ptr::null_mut(), Ordering::Release);
    }
}
```

## 持有檢查

```rust
unsafe fn holding(&self) -> bool {
    self.cpu.load(Ordering::Relaxed) == unsafe { proc::current_cpu() }
}

pub fn is_holding(&self) -> bool {
    let _intr_lock = proc::lock_current_cpu();
    unsafe { self.holding() }
}
```

## 中斷管理整合

```rust
pub struct InterruptLock;

pub struct Cpu {
    pub proc: Option<&'static Proc>,
    pub context: Context,
    pub num_off: isize,              // 鎖計數
    pub interrupts_enabled: bool,
}

impl Cpu {
    pub fn lock(&mut self, old_state: bool) -> InterruptLock {
        if self.num_off == 0 {
            self.interrupts_enabled = old_state;
        }
        self.num_off += 1;
        InterruptLock
    }

    pub fn unlock(&mut self) {
        self.num_off -= 1;
        if self.num_off == 0 && self.interrupts_enabled {
            interrupts::enable();
        }
    }
}
```

## 鎖定層級

```
lock_current_cpu()
    ↓
SpinLock::lock()
    ↓
中斷已停用（num_off 增加）
    ↓
取得自旋鎖
    ↓
操作受保護的資料
    ↓
釋放自旋鎖
    ↓
中斷恢復（num_off 減少）
```

## 為什麼需要停用中斷？

```rust
// 場景：程序 A 持有鎖 K
// 假設我們不禁用中斷

1. 中斷發生
2. 中斷處理常式嘗試取得鎖 K
3. 發生鎖死：
   - 中斷處理常式忙等待（因為鎖被程序 A 持有）
   - 程序 A 無法完成（因為 CPU 被中斷處理常式佔用）
```

## Panic 安全性

在除錯模式下，嘗試重新取得已持有的自旋鎖會 panic：

```rust
assert!(!self.holding(), "acquire spinlock {}", self.name);
```

## 執行緒安全

```rust
unsafe impl<T> Sync for SpinLock<T> where T: Send {}
unsafe impl<T> Send for SpinLock<T> where T: Send {}
unsafe impl<T> Sync for SpinLockGuard<'_, T> where T: Sync {}
```

## 與 SleepLock 的比較

| 特性 | SpinLock | SleepLock |
|------|----------|-----------|
| 等待方式 | 忙等待 | 睡眠 |
| 可用於中斷上下文 | 是 | 否 |
| 適用場景 | 短期鎖 | 長期鎖/ I/O |

## 相關主題

- [[sleeplock]]：睡眠鎖
- [[proc]]：程序管理與 CPU 鎖
- [[trap]]：中斷處理