# POSIX 標準概述

POSIX (Portable Operating System Interface) 是一系列定義作業系統 API 的標準，確保軟體在不同 Unix 系統間的可移植性。

## 歷史

| 年份 | 版本 | 說明 |
|------|------|------|
| 1988 | POSIX.1 | 基本 API（檔案、程序、錯誤）|
| 1990 | POSIX.1b | 即時擴充（POSIX.1b）|
| 1993 | POSIX.1c | 執行緒擴充 |
| 2001 | POSIX.1 | 整合多種擴充 |
| 2008 | POSIX.1-2008 | 最新主要版本 |

## xv8 支援的 POSIX API

xv8 實作了大量 POSIX API：

| 類別 | API |
|------|-----|
| 檔案 I/O | open, read, write, close, lseek |
| 程序管理 | fork, exec, wait, exit, getpid |
| 記憶體 | brk, mmap, munmap |
| 管道 | pipe |
| 訊號 | signal, kill |
| 時間 | gettimeofday, clock_gettime |
| 環境 | getenv, setenv |

## 與 xv6 的差異

xv8 從 xv6 (MIT 教學用 OS) 發展而來，但增加了許多 POSIX 相容性：

| 功能 | xv6 | xv8 |
|------|-----|-----|
| 系統呼叫數 | ~20 | ~50+ |
| 檔案系統 | 簡化 Unix v6 | 日誌式 Unix v7 |
| 網路 | 無 | 基本 UDP/TCP |
| 使用者程式 | 有限 | 完整 POSIX 工具 |

## POSIX 相容層

xv8-libc-compat 提供 POSIX 呼叫到 xv8 系統呼叫的橋接：

```rust
// 在主機上使用真正的 libc
#[cfg(not(target_arch = "riscv64"))]
real_libc = { package = "libc", version = "0.2" }

// 在 xv8 上使用自訂實作
#[cfg(target_arch = "riscv64")]
use xv8_libc::write;
```

## 標準化路徑

POSIX 定義標準路徑：
- `/bin`：基本命令
- `/usr/bin`：使用者命令
- `/tmp`：暫存檔案
- `/dev/null`：空裝置

xv8 的檔案系統遵循這些慣例。

## 錯誤碼

POSIX 定義標準錯誤碼：

| 編號 | 名稱 | 說明 |
|------|------|------|
| 1 | EPERM | 操作不允許 |
| 2 | ENOENT | 檔案不存在 |
| 5 | EIO | I/O 錯誤 |
| 9 | EBADF | 壞的檔案描述符 |
| 12 | ENOMEM | 記憶體不足 |
| 13 | EACCES | 權限被拒 |
| 17 | EEXIST | 檔案已存在 |
| 28 | ENOSPC | 空間不足 |

xv8 使用相同的錯誤碼。

## 符號連結 vs 硬連結

| 特性 | 符號連結 | 硬連結 |
|------|---------|--------|
| 目錄支援 | 可對目錄 | 不可對目錄 |
| 跨檔案系統 | 可 | 不可 |
| 刪除行為 | 指向失效 | 仍有效直到最後 |
| 儲存 | 路徑字串 | inode 計數 |

## POSIX 程序生命週期

```
fork() → 複製程序
    │
    ├─ 子: exec() → 執行新程式
    │           │
    │           └─ exit() → wait() 回收
    │
    └─ 父: wait() → 回收子程序
              │
              └─ 繼續執行或退出
```

## 檔案描述符

POSIX 使用小型非負整數作為檔案描述符：

| FD | 標準定義 |
|----|----------|
| 0 | 標準輸入 (stdin) |
| 1 | 標準輸出 (stdout) |
| 2 | 標準錯誤 (stderr) |

`open()` 返回下一個可用 FD。

## 檔案類型

POSIX 定義多種檔案類型：
- 一般檔案 (regular file)
- 目錄 (directory)
- 字元裝置 (character device)
- 區塊裝置 (block device)
- 管道 (pipe/FIFO)
- 符號連結 (symlink)
- 通訊端 (socket)

## 相關主題

- [[Process]]：程序管理
- [[Syscall]]：系統呼叫機制
- [[File-System]]：xv8 檔案系統