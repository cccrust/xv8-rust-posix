# Net — 網路抽象

`net.rs` 實作 `std::net` 模組，提供 TCP 與 UDP socket 的物件導向抽象。

## 實作的類型

- **TcpStream**: TCP 連線的讀寫串流，封裝 socket fd
- **TcpListener**: TCP 監聽器，接受連線請求
- **UdpSocket**: UDP socket，支援 `send_to`/`recv_from`

## 系統呼叫映射

| std::net 函式 | 系統呼叫 |
|-------------|---------|
| TcpStream::connect | socket + connect |
| TcpListener::bind | socket + bind + listen |
| TcpListener::accept | accept |
| UdpSocket::bind | socket + bind |
| UdpSocket::send_to | sendto |
| UdpSocket::recv_from | recvfrom |

## xv8 的適應

xv8 的 `std::net` 直接包裝核心網路系統呼叫，不經過任何標準 C 函式庫。由於 xv8 網路棧的設計限制，某些進階功能（如超時設定、SO_REUSEADDR、IPv6）可能不支援。

## 相關文件

- [net.md](../../net/src/net.md) — 網路功能模組
- [io.md](./io.md) — I/O 抽象
- [tcp.md](../../kernel/src/net/tcp.md) — TCP 協定
- [udp.md](../../kernel/src/net/udp.md) — UDP 協定
