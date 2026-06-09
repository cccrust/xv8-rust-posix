# Unix I/O 操作

提供 xv8 上的 Unix 風格 I/O 延伸操作。包括原始檔案描述子（RawFd）的複製（dup/dup2）、
檔案描述子旗標的讀取與設定（fcntl 的 F_GETFD/F_SETFD）、
非阻塞 I/O 模式設定（O_NONBLOCK）、以及檔案狀態旗標的查詢。
這些 API 對應 `std::os::unix::io` 模組的 `AsRawFd`、`FromRawFd`、`IntoRawFd` trait，
以及 `fcntl`、`dup` 等系統呼叫。在 xv8 核心上，這些操作直接透過
xv8-libc 的 syscall 包裝器發送給核心。
