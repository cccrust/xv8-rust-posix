# Unix 行程操作

提供 xv8 上的 Unix 行程管理延伸操作。包括行程群組（process group）的建立與管理
（`setpgid`、`getpgid`）、工作階段（session）管理（`setsid`、`getsid`）、
使用者與群組 ID 操作（`setuid`、`setgid`、`setreuid`、`setregid`、
`setresuid`、`setresgid`、`getresuid`、`getresgid`）、
補充群組（supplementary groups）操作（`getgroups`、`setgroups`、`initgroups`）。
這些 API 對應 Rust 標準程式庫的 `std::os::unix::process` 模組，
底層透過 xv8-libc 的 syscall 包裝器與 xv8 核心互動。
