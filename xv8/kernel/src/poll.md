# poll/epoll 模組 — poll.rs

## 理論背景

poll 與 epoll 是 Linux 的 I/O 多工 (I/O multiplexing) 機制，讓單一執行緒能同時監控多個檔案描述符的 I/O 事件。xv8 實作兩者。

- **poll** (POSIX 標準)：每次呼叫傳入完整的 fd 集合，核心掃描所有 fd 的狀態
- **epoll** (Linux 專有，2.5.44+)：使用事件驅動模型，fd 註冊後由核心主動通知

epoll 的性能優勢在於：
1. O(1) 就緒事件檢索（相較於 poll 的 O(N)）
2. 避免每次呼叫複製大量 fd 資料
3. 支援邊緣觸發 (edge-triggered) 與水平觸發 (level-triggered)

## xv8 實作

### poll

poll 遍歷 process 的開啟檔案表，為每個 fd 檢查可讀性與可寫性。xv8 的 `fd_readiness` 函數為每種 FileType 實作不同的就緒檢查邏輯。

### epoll

epoll 使用紅黑樹 (`RBTree`) 管理已註冊的 fd：

```rust
struct EpollEntry {
    fd: usize,
    events: u32,       // 感興趣的事件 EPOLIN|EPOLLOUT
    data: u64,         // 使用者資料
}
```

三種模式：
- **水平觸發 (LT)**: 只要 fd 仍可讀/寫，每次 `epoll_wait` 都會回傳
- **邊緣觸發 (ET)**: 僅在狀態變更時通知一次
- **獨佔 (EXCLUSIVE)**: 僅通知一個等待 process (Linux 4.5+)

### EpollEvent

```rust
pub struct EpollEvent {
    pub events: u32,   // 事件遮罩
    pub data: u64,     // 使用者資料
}
```

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 20 | `poll` | `(fds: *mut PollFd, nfds: usize, timeout: isize)` |
| 21 | `epoll_create1` | `(flags: u32)` |
| 22 | `epoll_ctl` | `(epfd: i32, op: i32, fd: i32, ev: *const EpollEvent)` |
| 23 | `epoll_wait` | `(epfd: i32, ev: *mut EpollEvent, max: i32, timeout: i32)` |

## 相關文件

- [syscall 文件](syscall.md)
- [file 文件](file.md)
