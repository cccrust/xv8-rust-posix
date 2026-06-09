# Udp — UDP 測試工具

`udp` 是在 xv8 上測試 UDP 傳輸的展示工具，用於驗證 UDP socket 的操作。

## UDP 測試展示

此工具展示 UDP 的無連線特性：

1. `socket(AF_INET, SOCK_DGRAM, 0)` — 建立 UDP socket
2. `sendto()` — 直接將資料報發送到目標位址，無需建立連線
3. `recvfrom()` — 從任意來源接收資料報（若綁定埠號後可接收回覆）

UDP 測試工具驗證：
- UDP 資料報的正確封裝與解封裝
- 網路層的正確路由
- 使用者/核心空間的資料複製

## 相關文件

- [udp.md](../../kernel/src/net/udp.md) — UDP 協定
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
