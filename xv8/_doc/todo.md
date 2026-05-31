# xv8 POSIX System Call 實作規劃

## 目前狀態 (v0.14)

已完成 v0.5~v0.14，已實作 **102 個** system calls。

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