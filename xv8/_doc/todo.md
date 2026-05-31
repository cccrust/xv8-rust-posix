# xv8 POSIX System Call 實作規劃

## 目前狀態 (v0.15)

已完成 v0.5~v0.15，已實作 **107 個** system calls。

### v0.16 - 

## 總計

從 54 個擴展到約 **112 個** system calls。

## 備註

- 網路相關 syscall（socket, connect, bind, listen, accept, send, receive 等）不在此規劃範圍
- 每個版本的具體實作順序可能根據需求調整
- 部分 syscall 可能需要先實作基礎設施（如 sigaction 需要 proc 支持）