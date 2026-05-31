# xv8 POSIX System Call 實作規劃

## 目前狀態 (v0.13)

已完成 v0.5~v0.13，已實作 **96 個** system calls。

### v0.14 - 檔案鎖定

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