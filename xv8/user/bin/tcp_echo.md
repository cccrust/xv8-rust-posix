# TCP Echo — TCP 回聲伺服器

`tcp_echo` 實作一個簡單的 TCP echo 伺服器，監聽指定埠號，將收到的任何資料原樣回傳給客戶端。

## Echo 伺服器模型

TCP echo 伺服器是網路程式設計中最基本的伺服器模型：

1. 建立 TCP socket
2. 綁定位址與埠號
3. 監聽連線請求
4. 接受連線，產生新的 socket
5. 從新 socket 讀取資料
6. 將同一資料寫回 socket
7. 關閉連線

## 教學意義

echo 伺服器展示 TCP 的雙向串流特性與 socket API 的使用，是測試網路協定棧正確性的標準方法。在 xv8 中，`tcp_echo` 可用於驗證 TCP 系統呼叫的三向交握、資料分段傳送與連線終止。

## 相關文件

- [tcpecho.md](../testbin/tcpecho.md) — TCP Echo 測試
- [tcp.md](../../kernel/src/net/tcp.md) — TCP 協定
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
