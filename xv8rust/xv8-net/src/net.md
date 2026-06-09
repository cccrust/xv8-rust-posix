# Net — 網路類型實作

`net.rs` 實作 `TcpStream`、`TcpListener`、`UdpSocket` 等網路類型，提供與 `std::net` 一致的 API。

## 實作策略

xv8-net 透過抽象層支援兩種後端：

### xv8 後端（riscv64）

當目標為 `riscv64gc-unknown-none-elf` 時：

- 所有網路操作直接使用 xv8-libc 的系統呼叫包裝
- `TcpStream` 封裝 fd + `connect` 系統呼叫
- `TcpListener` 封裝 fd + `bind`/`listen`/`accept` 系統呼叫
- `UdpSocket` 封裝 fd + `sendto`/`recvfrom` 系統呼叫

### Host 後端（非 riscv64）

當在主機上編譯（測試或除錯）時，所有類型委派給真正的 `std::net`。這讓 xv8-net 的程式可在開發機器上直接編譯測試。

## 非阻塞 I/O

xv8-net 支援 `set_nonblocking(true)` 設定，將 fd 標記為非阻塞模式。在此模式下，`read`/`write` 回傳 `WouldBlock` 錯誤而非阻塞等待。

## 相關文件

- [lib.md](./lib.md) — xv8-net 總覽
- [net.md](../../user-std/src/net.md) — xv8 使用者空間網路
- [raw.md](../../xv8-libc/src/raw.md) — 系統呼叫包裝
