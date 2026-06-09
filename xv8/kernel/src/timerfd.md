# timerfd 模組 — timerfd.rs

## 理論背景

timerfd 是 Linux 2.6.25 引入的計時器機制，將計時器到期事件轉換為檔案描述符的可讀事件。timerfd 與 `select`/`poll`/`epoll` 完全整合，是 POSIX 計時器 (`timer_create`) 的現代替代方案。

三種類型：
- `CLOCK_REALTIME`：系統即時時鐘 (可被 `settimeofday` 調整)
- `CLOCK_MONOTONIC`：單調時鐘 (不受系統時間調整影響)
- `CLOCK_BOOTTIME`：單調時鐘 + 睡眠時間 (Linux 3.15+)

## xv8 實作

### 資料結構

```rust
struct TimerFdState {
    clockid: u32,         // 時鐘類型
    itimer: Itimerspec,   // 初始間隔 + 重複間隔
    next_expiry: u64,     // 下次到期時間 (tick)
    interval: u64,        // 重複間隔 (tick)
    expired: u64,         // 到期次數 (read 回傳值)
    nonblock: bool,
}
```

### Itimerspec

```rust
pub struct Itimerspec {
    pub it_interval: Timespec,  // 重複間隔
    pub it_value: Timespec,     // 初始到期時間
}

pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}
```

### 操作

| 操作 | 行為 |
|------|------|
| `read()` | 回傳已到期次數 (`u64`)，歸零 counter |
| `write()` | 傳回 `BadDescriptor` |
| `TIMER_SETTIME` (ioctl) | 設定新計時器參數 |
| `TIMER_GETTIME` (ioctl) | 讀取目前計時器參數 |
| `poll()` | 到期次數 > 0 時可讀 |

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 36 | `timerfd_create` | `(clockid: i32, flags: u32)` |
| 37 | `timerfd_settime` | `(fd: i32, flags: u32, new: *const Itimerspec, old: *mut Itimerspec)` |
| 38 | `timerfd_gettime` | `(fd: i32, curr: *mut Itimerspec)` |

## 相關文件

- [syscall 文件](syscall.md)
- [trap 文件](trap.md) (TICKS 計時器來源)
