# Sysnet — 網路系統呼叫

## 概述

`sysnet` 模組實作網路相關的系統呼叫，作為使用者空間網路應用與核心網路協定棧之間的橋樑。這些系統呼叫包裝了 POSIX socket API 的核心部分。

## 實作的系統呼叫

| 系統呼叫 | 說明 |
|---------|------|
| `socket` | 建立 socket（domain, type, protocol），配置 `struct socket` 並關聯協定模組 |
| `bind` | 將 socket 綁定位址（IP + port），驗證權限與位址有效性 |
| `listen` | 將 TCP socket 設為監聽模式，建立連線佇列 |
| `accept` | 從連線佇列取出新連線，建立新 socket fd |
| `connect` | 發起連線請求（TCP 三向交握或 UDP 隱式連線） |
| `send` / `write` | 從使用者緩衝區複製資料，經 TCP/UDP 層封裝後傳送 |
| `recv` / `read` | 從 socket 接收佇列取出資料，複製到使用者緩衝區 |
| `close` | 關閉 socket，觸發 TCP FIN 或資源釋放 |

## 使用者/核心資料複製

網路系統呼叫需要在使用者空間與核心緩衝區之間複製封包資料。`sysnet` 使用 `copyin`/`copyout` 函式安全地進行複製，避免使用者空間的惡意指標存取核心記憶體。

## 實作流程

```
User: fd = socket(AF_INET, SOCK_STREAM, 0)
  → syscall(a7=SYS_SOCKET, a0=AF_INET, a1=SOCK_STREAM, a2=0)
    → sysnet::socket()
      → find free fd slot
      → allocate socket + TCP control block
      → return fd to user
```

## 相關文件

- [net/mod.md](../net/mod.md) — 網路協定棧總覽
- [tcp.md](../net/tcp.md) — TCP 協定實作
- [udp.md](../net/udp.md) — UDP 協定實作
- [socket.md](../net/socket.md) — Socket 層抽象
