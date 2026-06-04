# 睡眠鎖 — sleeplock.rs

睡眠鎖允許在等待鎖時讓出 CPU，適用於需要長時間 I/O 等待的場景。

## 與自旋鎖的比較

| 特性 | SpinLock | SleepLock |
|------|----------|-----------|
| 等待方式 | 忙等待 | 讓出 CPU |
| 持有期間可睡眠 | 否 | 是 |
| 中斷狀態 | 停用中斷 | 保持中斷 |
| 適用場景 | 短期鎖 | 長期鎖、I/O |

## 資料結構

```rust
pub struct SleepLock<T> {
    _name: &'static str,
    // 內部用 SpinLock 保護鎖狀態
    inner: SpinLock<SleepLockInner>,
    data: UnsafeCell<T>,
}

struct SleepLockInner {
    locked: bool,           // 鎖是否被持有
    pid: Option<Pid>,      // 持有者的 PID
}
```

## 獲取鎖

```rust
pub fn lock(&self) -> SleepLockGuard<'_, T> {
    let mut inner = self.inner.lock();

    while inner.locked {
        // 釋放 SpinLock，進入睡眠
        inner = proc::sleep(Channel::Lock(self as *const _ as usize), inner);
    }

    inner.locked = true;
    inner.pid = Some(proc::current_proc().inner.lock().pid);

    SleepLockGuard { lock: self }
}
```

## 釋放鎖

```rust
impl Drop for SleepLockGuard<'_, T> {
    fn drop(&mut self) {
        let mut inner = self.lock.inner.lock();
        inner.locked = false;
        inner.pid = None;

        // 喚醒等待者
        proc::wakeup(Channel::Lock(self.lock as *const _ as usize));
    }
}
```

## Guard 行為

```rust
impl<T> Deref for SleepLockGuard<'_, T> {
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SleepLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
```

## 使用範例

inode 鎖定：
```rust
pub fn lock(&self) -> SleepLockGuard<'static, InodeInner> {
    let mut inner = INODE_TABLE.inner[self.id].lock();

    if !inner.valid {
        // 從磁碟讀取 inode
        let buf = BCACHE.read(self.dev, ...);
        // ...
    }

    inner
}
```

## 執行緒安全

```rust
// SleepLock 本身是 Sync（只要 T: Send）
unsafe impl<T> Sync for SleepLock<T> where T: Send {}

// SleepLock 本身是 Send（只要 T: Send）
unsafe impl<T> Send for SleepLock<T> where T: Send {}

// 但 SleepLockGuard 不是 Sync
// 這是正確的，因為 guard 只能被單一執行緒持有
```

## 與 SpinLock 的組合使用

在 inode 的場景：
1. `INODE_TABLE.meta` 使用 `SpinLock`（保護引用計數、快速查找）
2. `INODE_TABLE.inner` 使用 `SleepLock`（保護實際 inode 資料、允许長 I/O）

```rust
struct InodeTable {
    meta: SpinLock<[InodeMeta; NINODE]>,      // 快速路徑
    inner: [SleepLock<InodeInner>; NINODE],   // 慢速路徑
}
```

## 與 proc::sleep 的整合

SleepLock 使用 `Channel::Lock` 來區分不同的鎖：

```rust
proc::sleep(Channel::Lock(self as *const _ as usize), inner)
```

這允許精確喚醒特定鎖的等待者，而不是喚醒所有睡眠的程序。

## 相關主題

- [[spinlock]]：自旋鎖
- [[fs]]：檔案系統 inode
- [[Process]]：程序睡眠/喚醒