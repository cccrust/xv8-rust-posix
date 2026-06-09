# Tcpecho — TCP Echo 測試

`tcpecho` 測試驗證 xv8 的 TCP echo 伺服器功能，確認 TCP 連線建立、資料傳送、接收與連線終止的完整流程。測試啟動 TCP echo server 與 client，驗證三向交握（SYN/SYN-ACK/ACK）的正常運作、雙向資料串流的正確傳遞、TCP segment 的序號管理與連線關閉（FIN）的四次揮手程序。

## 相關文件

- [tcp.md](../../kernel/src/net/tcp.md) — TCP 協定
- [tcp_echo.md](../bin/tcp_echo.md) — TCP Echo 伺服器
- [sysnet.md](../../kernel/src/sysnet.md) — 網路系統呼叫
