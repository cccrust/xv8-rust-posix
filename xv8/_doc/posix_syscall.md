# xv8 POSIX System Call 支援分析

## 已實作的 System Calls (89 個)

| syscall | 狀態 | 說明 |
|---------|------|------|
| fork | ✓ | POSIX 標準 |
| exit | ✓ | POSIX 標準 |
| wait | ✓ | POSIX 標準 (wait/waitpid) |
| pipe | ✓ | POSIX 標準 |
| read | ✓ | POSIX 標準 |
| write | ✓ | POSIX 標準 |
| kill | ✓ | POSIX 標準 |
| exec | ✓ | 類似 execve |
| fstat | ✓ | POSIX 標準 |
| chdir | ✓ | POSIX 標準 |
| dup | ✓ | POSIX 標準 |
| getpid | ✓ | POSIX 標準 |
| sbrk | ✓ | 非標準，但類似 brk |
| sleep | ✓ | 類似 nanosleep |
| uptime | ✓ | 非標準 |
| open | ✓ | POSIX 標準 |
| mknod | ✓ | POSIX 標準 |
| unlink | ✓ | POSIX 標準 |
| link | ✓ | POSIX 標準 |
| mkdir | ✓ | POSIX 標準 |
| close | ✓ | POSIX 標準 |
| poweroff | ✓ | 非標準 (xv8 專有) |
| ioctl | △ | 部分支援 |
| socket/send/receive | ✓ | BSD socket API |
| lseek | ✓ | POSIX 標準 (新增) |
| truncate | ✓ | POSIX 標準 (新增) |
| ftruncate | ✓ | POSIX 標準 (新增) |
| chmod | ✓ | POSIX 標準 (新增) |
| fchmod | ✓ | POSIX 標準 (新增) |
| chown | ✓ | POSIX 標準 (新增) |
| fchown | ✓ | POSIX 標準 (新增) |
| access | ✓ | POSIX 標準 (新增) |
| rename | ✓ | POSIX 標準 (新增) |
| umask | ✓ | POSIX 標準 (新增) |
| getuid | ✓ | POSIX 標準 (新增) |
| geteuid | ✓ | POSIX 標準 (新增) |
| getgid | ✓ | POSIX 標準 (新增) |
| getegid | ✓ | POSIX 標準 (新增) |
| gettimeofday | ✓ | POSIX 標準 (新增) |
| uname | ✓ | POSIX 標準 (新增) |
| setpgid | ✓ | POSIX 標準 (v0.9) |
| getsid | ✓ | POSIX 標準 (v0.9) |
| setreuid | ✓ | POSIX 標準 (v0.9) |
| setregid | ✓ | POSIX 標準 (v0.9) |
| setresuid | ✓ | POSIX 標準 (v0.9) |
| setresgid | ✓ | POSIX 標準 (v0.9) |
| getresuid | ✓ | POSIX 標準 (v0.9) |
| getresgid | ✓ | POSIX 標準 (v0.9) |
| mkfifo | ✓ | POSIX 標準 (v0.10) |
| pipe2 | ✓ | POSIX 標準 (v0.10) |
| ttyname | ✓ | POSIX 標準 (v0.11) |
| ttyioctl | ✓ | POSIX 標準 (v0.11) |
| tcgetsid | ✓ | POSIX 標準 (v0.11) |
| tcflow | ✓ | POSIX 標準 (v0.11) |
| tcflush | ✓ | POSIX 標準 (v0.11) |
| pathconf | ✓ | POSIX 標準 (v0.12) |
| fpathconf | ✓ | POSIX 標準 (v0.12) |
| sysconf | ✓ | POSIX 標準 (v0.12) |
| confstr | ✓ | POSIX 標準 (v0.12) |
| setgroups | ✓ | POSIX 標準 (v0.13) |
| getgroups | ✓ | POSIX 標準 (v0.13) |
| initgroups | ✓ | POSIX 標準 (v0.13) |
| sigaction | ✓ | POSIX 標準 (v0.14) |
| sigprocmask | ✓ | POSIX 標準 (v0.14) |
| sigpending | ✓ | POSIX 標準 (v0.14) |
| sigsuspend | ✓ | POSIX 標準 (v0.14) |
| sigreturn | ✓ | POSIX 標準 (v0.14) |
| killpg | ✓ | POSIX 標準 (v0.14) |
| getenv | ✓ | POSIX 標準 (v0.15) |
| setenv | ✓ | POSIX 標準 (v0.15) |
| unsetenv | ✓ | POSIX 標準 (v0.15) |
| clearenv | ✓ | POSIX 標準 (v0.15) |
| getpagesize | ✓ | POSIX 標準 (v0.15) |

## OpenFlag POSIX 標準値

```rust
// 已修正為 POSIX 標準:
O_RDONLY   = 0x000
O_WRONLY   = 0x001
O_RDWR     = 0x002
O_CREAT    = 0x040
O_EXCL     = 0x080
O_TRUNC    = 0x200
O_APPEND   = 0x400
O_NONBLOCK = 0x800
```

## Stat 結構已擴展

```rust
pub struct Stat {
    pub dev: u32,
    pub ino: u32,
    pub r#type: InodeType,
    pub mode: u16,      // 新增
    pub nlink: u16,
    pub uid: u16,       // 新增
    pub gid: u16,       // 新增
    pub size: u64,
    pub atime: u32,     // 新增
    pub mtime: u32,     // 新增
}
```

## 尚未實作的 System Calls

| syscall | 說明 |
|---------|------|
| stat / fstatat | 已有 fstat，stat 可透過路徑實現 |
| symlink / readlink | 符號連結支援 |
| utimensat | 改變檔案時間戳 |
| mount / umount | 掛載/卸載檔案系統 |

## 測試狀態

所有 8 個測試通過 (v0.11):
- test fs ... ok
- test pipe ... ok
- test proc ... ok
- test fd ... ok
- test sbrk ... ok
- test cow ... ok
- test net ... ok
- test syscall ... ok

## 結論

xv8 已支援大多數常用的 POSIX system calls，包括:
- 基礎檔案操作 (read, write, open, close, lseek, truncate, chmod, chown, rename)
- 程序管理 (fork, exec, exit, wait, kill, getpid)
- 目錄操作 (mkdir, rmdir, chdir)
- 連結操作 (link, unlink)
- 時間查詢 (gettimeofday, uptime, sleep, time, nanosleep, clock_gettime)
- 身份識別 (getuid, geteuid, getgid, getegid, setreuid, setregid, setresuid, setresgid, getresuid, getresgid, setuid, setgid)
- 程序群組 (getpgid, setpgid, getpgrp, setsid, getsid)
- 命名管道 (mkfifo, pipe2)
- 系統資訊 (uname)
- 網路 socket (socket, send, receive)
- 記憶體 (mmap, munmap, mprotect, sbrk)
- 終端 (isatty, tcgetattr, tcsetattr, ttyname, ttyioctl, tcgetsid, tcflow, tcflush)