# Process 系統呼叫 — sysproc.rs

## 概述

`sysproc.rs` 實作與 process 管理相關的系統呼叫，包括：行程控制、記憶體管理、檔案系統操作、訊號處理、排程控制、容器隔離等。

xv8 的系統呼叫總數超過 110 個。`sysproc.rs` 負責其中約 70 個，其餘由 `sysfile.rs`（檔案操作）、`sysnet.rs`（網路操作）處理。

## 系統呼叫分類

### Process 控制

| 編號 | 名稱 | 說明 |
|------|------|------|
| 0 | `fork` | 建立子 process |
| 1 | `exit` | 終止 process |
| 2 | `wait` | 等待子 process |
| 7 | `exec` | 載入並執行新程式 |
| 8 | `waitpid` | 等待指定子 process |
| 10 | `getpid` | 取得 PID |
| 11 | `getppid` | 取得父 PID |
| 12 | `getpgid` | 取得行程群組 ID |
| 13 | `setpgid` | 設定行程群組 ID |
| 14 | `getsid` | 取得 session ID |
| 15 | `setsid` | 建立新 session |
| 25 | `sched_yield` | 自願釋放 CPU |
| 26 | `clone` | 建立執行緒 |
| 40 | `nanosleep` | 高精度睡眠 |
| 41 | `getpagesize` | 取得頁面大小 |

### 記憶體管理

| 編號 | 名稱 | 說明 |
|------|------|------|
| 17 | `sbrk` | 調整 heap |
| 18 | `mmap` | 記憶體映射 |
| 19 | `munmap` | 解除映射 |
| 20 | `mprotect` | 變更保護屬性 |

### 檔案系統

| 編號 | 名稱 | 說明 |
|------|------|------|
| 16 | `open` | 開啟檔案 |
| 21 | `read` | 讀取檔案 |
| 22 | `write` | 寫入檔案 |
| 23 | `close` | 關閉檔案 |
| 24 | `unlink` | 刪除檔案 |
| 24 | `link` | 建立硬連結 |
| 27 | `mkdir` | 建立目錄 |
| 28 | `rmdir` | 刪除目錄 |
| 29 | `chdir` | 變更工作目錄 |
| 30 | `mknod` | 建立裝置節點 |
| 31 | `chmod` | 變更權限 |
| 38 | `fstat` | 取得檔案狀態 |
| 39 | `stat` | 取得檔案狀態 |
| 44 | `dup` | 複製 fd |
| 45 | `dup2` | 複製 fd 到指定號碼 |
| 47 | `lseek` | 檔案定位 |

### 訊號處理

| 編號 | 名稱 | 說明 |
|------|------|------|
| 48 | `sigaction` | 設定訊號處理器 |
| 49 | `sigprocmask` | 設定訊號遮罩 |
| 50 | `sigpending` | 取得待處理訊號 |
| 51 | `sigsuspend` | 等待訊號 |
| 52 | `sigreturn` | 從訊號處理器返回 |
| 53 | `kill` | 發送訊號 |
| 54 | `killpg` | 發送訊號到群組 |
| 55 | `signal` | 簡易訊號設定 |

### 使用者與群組

| 編號 | 名稱 | 說明 |
|------|------|------|
| 56 | `getuid` | 取得 UID |
| 57 | `getgid` | 取得 GID |
| 58 | `setuid` | 設定 UID |
| 59 | `setgid` | 設定 GID |
| 60 | `setreuid` | 設定真實/有效 UID |
| 61 | `setregid` | 設定真實/有效 GID |
| 62 | `setresuid` | 設定三種 UID |
| 63 | `setresgid` | 設定三種 GID |
| 64 | `getresuid` | 取得三種 UID |
| 65 | `getresgid` | 取得三種 GID |
| 66 | `getgroups` | 取得輔助群組 |
| 67 | `setgroups` | 設定輔助群組 |
| 68 | `initgroups` | 初始化輔助群組 |

### Time

| 編號 | 名稱 | 說明 |
|------|------|------|
| 69 | `time` | 取得目前時間 |
| 70 | `clock_gettime` | 取得時鐘時間 |
| 71 | `gettimeofday` | 取得時間與時區 |

### 容器

| 編號 | 名稱 | 說明 |
|------|------|------|
| 140 | `unshare` | 建立新 namespace |
| 141 | `setns` | 加入 namespace |
| 142 | `capget` | 取得能力 |
| 143 | `capset` | 設定能力 |
| 144 | `seccomp` | 安全計算過濾器 |
| 145 | `pivot_root` | 切換 root |
| 146 | `sethostname` | 設定主機名稱 |
| 147 | `gethostname` | 取得主機名稱 |
| 148 | `overlay_mount` | 掛載 OverlayFS |
| 149 | `overlay_umount` | 卸載 OverlayFS |
| 150 | `nsopen` | 開啟 namespace fd |

## 實作模式

多數系統呼叫遵循相同模式：

1. 從 `SyscallArgs` 解析參數
2. 驗證參數（檢查指標、範圍、權限）
3. 執行操作
4. 回傳結果或 `SysError`

```rust
pub fn sys_getpid(_args: &SyscallArgs) -> Result<usize, SysError> {
    let proc = current_proc();
    Ok(*proc.inner.lock().pid)
}
```

## 相關文件

- [syscall 文件](syscall.md)
- [proc 文件](proc.md)
- [Wiki: Namespace](../../../_wiki/Namespace.md)
- [Wiki: 容器](../../../_wiki/Container.md)
