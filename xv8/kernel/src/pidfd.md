# pidfd 模組 — pidfd.rs

## 理論背景

pidfd (process ID file descriptor) 是 Linux 5.3 引入的機制，建立一個與 process 生命週期綁定的檔案描述符。傳統的 PID 可被回收再利用 (PID reuse)，導致競爭條件 (race condition)。pidfd 提供了一個穩定的 process 參照──只要 fd 未關閉，參照的 process 就保證指向原始 process。

## xv8 實作

### 資料結構

```rust
struct PidFdState {
    target_pid: usize,    // 目標 process PID
    alive: bool,          // process 是否仍存活
}
```

### 操作

| 操作 | 行為 |
|------|------|
| `read()` | 傳回 `BadDescriptor` (pidfd 不可讀取) |
| `write()` | 傳回 `BadDescriptor` (pidfd 不可寫入) |
| `poll()` | 若 process 已結束，標記為可讀 |
| close | 清除狀態，fd 關閉不影響目標 process |

### pidfd_is_alive

```rust
pub fn pidfd_is_alive(pidfd_id: usize) -> bool;
```

檢查目標 process 是否仍在執行。

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 34 | `pidfd_open` | `(pid: usize, flags: u32)` |

## 相關文件

- [syscall 文件](syscall.md)
- [proc 文件](proc.md)
