# 檔案系統呼叫 — sysfile.rs

## 概述

`sysfile.rs` 實作與檔案操作相關的系統呼叫包裝器，包括：開啟、關閉、讀取、寫入、目錄操作、檔案資訊查詢等。

## 系統呼叫

| 編號 | 名稱 | 原型 |
|------|------|------|
| 16 | `open` | `(path: *const u8, flags: usize, mode: usize)` |
| 21 | `read` | `(fd: i32, buf: *mut u8, count: usize)` |
| 22 | `write` | `(fd: i32, buf: *const u8, count: usize)` |
| 23 | `close` | `(fd: i32)` |
| 24 | `unlink` | `(path: *const u8)` |
| 37 | `link` | `(old: *const u8, new: *const u8)` |
| 38 | `fstat` | `(fd: i32, stat: *mut Stat)` |
| 39 | `stat` | `(path: *const u8, stat: *mut Stat)` |
| 44 | `dup` | `(fd: i32)` |
| 45 | `dup2` | `(old: i32, new: i32)` |
| 47 | `lseek` | `(fd: i32, offset: isize, whence: i32)` |
| 70 | `readv` | `(fd: i32, iov: *const IoVec, iovcnt: i32)` |
| 71 | `writev` | `(fd: i32, iov: *const IoVec, iovcnt: i32)` |
| 72 | `pread` | `(fd: i32, buf: *mut u8, count: usize, offset: usize)` |
| 73 | `pwrite` | `(fd: i32, buf: *const u8, count: usize, offset: usize)` |

### fd_alloc / fd 管理

`fd_alloc(file)` 為 process 分配一個新的 fd 編號，將 `File` 插入 process 的 fd 表。

所有檔案操作系統呼叫都透過 `SyscallArgs::get_file()` 將 fd 編號轉換為 `File` 物件，再由 `File` 物件分派到對應的 `FileType` 處理。

## 相關文件

- [syscall 文件](syscall.md)
- [file 文件](file.md)
- [fs 文件](fs.md)
- [sysproc 文件](sysproc.md)
