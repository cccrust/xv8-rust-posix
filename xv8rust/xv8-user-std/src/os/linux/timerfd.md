# Linux timerfd 包裝器

提供 `timerfd_create()`、`timerfd_settime()`、`timerfd_gettime()` 系統呼叫的
Rust 包裝。timerfd 將定時器表示為檔案描述子，可與 `select()`、`poll()`、
`epoll()` 整合使用。支援 `CLOCK_REALTIME`（系統即時時鐘）、
`CLOCK_MONOTONIC`（單調遞增時鐘，不受系統時間調整影響）、
`TFD_NONBLOCK`、`TFD_CLOEXEC` 等選項。timerfd 的設計哲學是
"一切都是檔案描述子"——定時器事件可與 I/O 事件統一處理，
簡化了事件驅動程式設計的非同步定時需求。
