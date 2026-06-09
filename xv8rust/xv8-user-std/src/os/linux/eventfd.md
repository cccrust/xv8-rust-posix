# Linux eventfd 包裝器

提供 `eventfd()` 系統呼叫的 Rust 包裝。eventfd 是一個輕量級事件通知機制，
建立一個核心管理的 64 位元計數器檔案描述子。行程可透過 `read()`/`write()`
或 `select()`/`poll()`/`epoll()` 等待計數器非零。常見用途：
行程間事件通知、非同步 I/O 完成通知、避免鎖的輕量級同步。
在 xv8 核心中，eventfd 作為核心的內部事件機制，對應 `EFD_NONBLOCK`、
`EFD_SEMAPHORE`（訊號量模式，每次讀取減 1）等旗標。
