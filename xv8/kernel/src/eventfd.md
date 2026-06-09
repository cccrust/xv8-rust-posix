# eventfd 模組 — eventfd.rs

## 理論背景

eventfd 是 Linux 2.6.22 引入的輕量級事件通知機制，建立一個可供 `read`/`write` 操作的檔案描述符，用於使用者空間與核心空間之間的事件信號傳遞。

eventfd 的值是一個 64 位元無號整數 (`u64`)，檔案描述符與 `EFD_SEMAPHORE`、`EFD_NONBLOCK`、`EFD_CLOEXEC` 旗標一同建立。

## xv8 實作

### 資料結構

```rust
struct EventFdState {
    value: u64,           // 計數器值
    semaphore: bool,      // EFD_SEMAPHORE 模式
}
```

全域 `EVENTFD_TABLE` 管理所有 eventfd 實例。

### 語意

| 操作 | 一般模式 | Semaphore 模式 |
|------|---------|---------------|
| `write(val)` | `value += val` | `value += val` |
| `read()` | 回傳 `value`，設為 0 | 回傳 1，`value -= 1` |
| `poll` | `value > 0` 時可讀 | `value > 0` 時可讀 |

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 32 | `eventfd2` | `(initval: u32, flags: u32)` |

## 相關文件

- [syscall 文件](syscall.md)
