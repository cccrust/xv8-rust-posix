# std::net

網路操作，net/ 工具使用。

## TcpListener

伺服器監聽：

```rust
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:8080")?;
println!("Listening on port 8080");

for stream in listener.incoming() {
    let stream = stream?;
    // 處理連接
}
```

## TcpStream

客戶端連接：

```rust
use std::net::TcpStream;

let mut stream = TcpStream::connect("127.0.0.1:8080")?;
stream.write_all(b"Hello")?;
```

### 讀寫

```rust
use std::io::{Read, Write};

let mut stream = TcpStream::connect("127.0.0.1:8080")?;
stream.write_all(b"GET / HTTP/1.0\r\n\r\n")?;

let mut buffer = vec![0u8; 1024];
stream.read(&mut buffer)?;
```

### set_read_timeout / set_write_timeout

```rust
use std::time::Duration;

stream.set_read_timeout(Some(Duration::from_secs(30)))?;
stream.set_write_timeout(Some(Duration::from_secs(30)))?;
```

## UdpSocket

UDP 通信：

```rust
use std::net::UdpSocket;

let socket = UdpSocket::bind("127.0.0.1:0")?;
socket.send_to(b"Hello", "127.0.0.1:8080")?;

let mut buf = [0u8; 1024];
let (size, src) = socket.recv_from(&mut buf)?;
```

## SocketAddr

位址表示：

```rust
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};

let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
let addr = SocketAddr::new(ip, 8080);
```

## IpAddr / Ipv4Addr / Ipv6Addr

```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

let ipv4: IpAddr = Ipv4Addr::new(192, 168, 1, 1).into();
let ipv6: IpAddr = Ipv6Addr::new(
    0, 0, 0, 0, 0, 0xffff, 0xc0a8_0101
).into();
```

## ToSocketAddrs

域名解析：

```rust
use std::net::{TcpListener, ToSocketAddrs};

for stream in listener.incoming() {
    // ...
}
```

## 本專案使用

### traceroute

```rust
use std::net::{UdpSocket, SocketAddr};

let socket = UdpSocket::bind("0.0.0.0:0")?;
socket.send_to(&packet, &addr)?;
let mut buf = [0u8; 512];
let (size, src) = socket.recv_from(&mut buf)?;
```

### DNS lookup

```rust
use std::net::{UdpSocket, SocketAddr};

let socket = UdpSocket::bind("0.0.0.0:0")?;
socket.send_to(&query, &dns_server)?;
socket.recv_from(&mut response)?;
```

### HTTP server

```rust
use std::net::{TcpListener, TcpStream};

let listener = TcpListener::bind("0.0.0.0:80")?;
for stream in listener.incoming() {
    let stream = stream?;
    thread::spawn(|| handle_connection(stream));
}
```

### ping (ICMP)

```rust
use std::net::IcmpSocket;  // 注意：std 沒有直接的 ICMP socket
```

## 錯誤處理

```rust
use std::io;
use std::net;

match TcpStream::connect("invalid:99999") {
    Ok(s) => { /* 成功 */ }
    Err(e) => eprintln!("Connection failed: {}", e),
}
```

## shutdown

```rust
use std::net::Shutdown;

stream.shutdown(Shutdown::Read)?;   // 關閉讀
stream.shutdown(Shutdown::Write)?;  // 關閉寫
stream.shutdown(Shutdown::Both)?;   // 兩者都關閉
```

## 底層機制

- **Linux/macOS**：Berkeley sockets API
- **xv8**：透過 syscall 的網路堆疊

## Socket 選項

```rust
use std::net::TcpKeepAlive;

stream.set_keepalive(Some(TcpKeepAlive {
    time: Duration::from_secs(30),
    interval: Some(Duration::from_secs(10)),
    retries: Some(3),
}))?;
```

## 本機位址 / 遠端位址

```rust
let local = stream.local_addr()?;
let remote = stream.peer_addr()?;
```

## 與 POSIX 的對應

| Rust | POSIX |
|------|-------|
| `TcpStream::connect` | `socket` + `connect` |
| `TcpListener::bind` | `socket` + `bind` + `listen` |
| `UdpSocket` | `SOCK_DGRAM` socket |
| `read`/`write` | `recv`/`send` |

## 相關模組

- `std::io`：I/O 操作
- `std::time`：超時
- `std::sync`：同步