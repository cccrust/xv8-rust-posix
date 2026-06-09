# signalfd 模組 — signalfd.rs

## 理論背景

signalfd 是 Linux 2.6.22 引入的機制，將訊號傳遞 (signal delivery) 轉換為檔案描述符的可讀事件。傳統的訊號處理使用 signal handler callback，在非同步的訊號上下文中執行，限制很多（只能呼叫 async-signal-safe 函數）。signalfd 讓 process 可以用 `read()` 從 fd 讀取 `SignalfdSiginfo` 結構，在一般執行緒上下文中處理訊號。

## xv8 實作

### 資料結構

每個 signalfd 關聯一個訊號遮罩 (signal mask)，表示此 fd 接收哪些訊號：

```rust
struct SignalfdState {
    mask: u32,               // 訊號遮罩
    pending: Vec<Siginfo>,   // 待處理訊號佇列
}
```

### SignalfdSiginfo

```rust
pub struct SignalfdSiginfo {
    pub signo: u32,     // 訊號編號
    pub errno: i32,     // 錯誤碼
    pub code: i32,      // 訊號碼 (SI_*)
    pub pid: u32,       // 發送 process 的 PID
    pub uid: u32,       // 發送 process 的 UID
    pub fd: i32,        // 檔案描述符 (SIGIO)
    pub tid: u32,       // 目標執行緒 ID
    pub band: u32,      // 頻寬事件 (SIGIO)
    pub overrun: u32,   // POSIX timer overrun count
    pub trapno: u32,    // 陷阱編號
    pub status: i32,    // exit status (SIGCHLD)
    pub utime: i64,     // 使用者時間
    pub stime: i64,     // 系統時間
    pub addr: u64,      // 錯誤位址 (SIGSEGV, SIGBUS)
}
```

### 操作

| 操作 | 行為 |
|------|------|
| `read()` | 從 pending 佇列取出 `SignalfdSiginfo` 結構 |
| `write()` | 傳回 `BadDescriptor` |
| `poll()` | pending 佇列非空時可讀 |

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 35 | `signalfd4` | `(fd: i32, mask: *const u32, flags: u32)` |

## 相關文件

- [syscall 文件](syscall.md)
- [signal 文件](signal.md)
