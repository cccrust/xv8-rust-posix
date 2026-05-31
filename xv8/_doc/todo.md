# xv8 POSIX System Call 實作規劃

## 目前狀態 (v0.5)

已完成 v0.5，已實作 **62 個** system calls。

## 待實作項目

### v0.6 - 記憶體管理

| syscall | 編號 | 說明 |
|---------|------|------|
| mmap | 62 | 建立記憶體映射 |
| munmap | 63 | 解除記憶體映射 |
| mprotect | 64 | 設定記憶體保護 |
| brk | 65 | 改變資料段大小 |

### v0.7 - 時間與計時器

| syscall | 編號 | 說明 |
|---------|------|------|
| time | 66 | 取得時間（秒） |
| nanosleep | 67 | 高精度睡眠 |
| clock_gettime | 68 | 取得時鐘時間 |
| clock_getres | 69 | 取得時鐘解析度 |
| clock_settime | 70 | 設定時鐘時間 |

### v0.8 - 分散式 I/O

| syscall | 編號 | 說明 |
|---------|------|------|
| readv | 71 | 分散式讀取 |
| writev | 72 | 分散式寫入 |
| pread | 73 | 指定偏移讀取 |
| pwrite | 74 | 指定偏移寫入 |

### v0.9 - 程序群組與 session

| syscall | 編號 | 說明 |
|---------|------|------|
| setpgid | 75 | 設定程序群組（跨程序） |
| getsid | 76 | 取得 session ID |
| setreuid | 77 | 設定真實/有效 UID |
| setregid | 78 | 設定真實/有效 GID |
| setresuid | 79 | 設定真實/有效/儲存 UID |
| setresgid | 80 | 設定真實/有效/儲存 GID |
| getresuid | 81 | 取得真實/有效/儲存 UID |
| getresgid | 82 | 取得真實/有效/儲存 GID |

### v0.10 - FIFO 與管道增強

| syscall | 編號 | 說明 |
|---------|------|------|
| mkfifo | 83 | 建立命名管道 |
| pipe2 | 84 | 建立管道（帶 flags） |

### v0.11 - TTY 擴展

| syscall | 編號 | 說明 |
|---------|------|------|
| ttyname | 85 | 取得終端名稱 |
| ttyioctl | 86 | TTY 控制操作 |
| tcgetsid | 87 | 取得 session ID |
| tcflow | 88 | 終端流量控制 |
| tcflush | 89 | 清除終端緩衝 |

### v0.12 - 路徑與系統配置

| syscall | 編號 | 說明 |
|---------|------|------|
| pathconf | 90 | 查詢路徑配置 |
| fpathconf | 91 | 查詢 fd 配置 |
| sysconf | 92 | 查詢系統配置 |
| confstr | 93 | 查詢字串配置 |

### v0.13 - 群組管理

| syscall | 編號 | 說明 |
|---------|------|------|
| setgroups | 94 | 設定附加群組 |
| getgroups | 95 | 取得群組列表 |
| initgroups | 96 | 初始化群組 |

### v0.14 - 信號增強

| syscall | 編號 | 說明 |
|---------|------|------|
| sigaction | 97 | 設定信號處理 |
| sigprocmask | 98 | 設定信號遮罩 |
| sigpending | 99 | 取得待處理信號 |
| sigsuspend | 100 | 原子替換遮罩並等待 |
| sigreturn | 101 | 從信號處理返回 |
| killpg | 102 | 發送信號到程序群組 |

### v0.15 - 環境與參數

| syscall | 編號 | 說明 |
|---------|------|------|
| getenv | 103 | 取得環境變數 |
| setenv | 104 | 設定環境變數 |
| unsetenv | 105 | 移除環境變數 |
| clearenv | 106 | 清除環境 |
| getpagesize | 107 | 取得記憶體頁大小 |

## 總計

從 54 個擴展到約 **107 個** system calls。

## 備註

- 網路相關 syscall（socket, connect, bind, listen, accept, send, receive 等）不在此規劃範圍
- 每個版本的具體實作順序可能根據需求調整
- 部分 syscall 可能需要先實作基礎設施（如 sigaction 需要 proc 支持）