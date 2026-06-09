# Net Impl — 網路工具實作

`net_impl.rs` 提供跨平台的網路 socket 操作包裝，統一 macOS 與 Linux 的 socket API 差異。

## 平台抽象

網路程式設計在 POSIX 系統上大致一致，但細節有差異：

- **macOS**: `SO_NOSIGPIPE` 選項避免 SIGPIPE 訊號
- **Linux**: `MSG_NOSIGNAL` 標誌達成相同效果
- **ICMP socket**: macOS 使用 `SOCK_DGRAM` + IPPROTO_ICMP，Linux 需要 `SOCK_RAW` + 權限

`net_impl.rs` 透過條件編譯處理這些差異，使上層工具程式碼保持平台無關。

## 封裝的函式

- `create_icmp_socket()`: 建立 ICMP socket
- `create_udp_socket()`: 建立 UDP socket
- `create_tcp_socket()`: 建立 TCP socket
- `set_socket_timeout()`: 設定接收/傳送超時
- `resolve_hostname()`: 透過系統 getaddrinfo 解析主機名稱

## 相關文件

- [lib.md](./lib.md) — libnet 總覽
- [util.md](./util.md) — 工具函式
