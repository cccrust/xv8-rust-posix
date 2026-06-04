# ping — ICMP Echo 測試

ping 傳送 ICMP Echo Request 並等待 Echo Reply，測量網路連線和延遲。

## 使用方式

```bash
ping <host> [count]
```

## 輸出範例

```
PING 10.0.2.2 32 bytes of data
64 bytes from 10.0.2.2: icmp_seq=1 time=10ms
64 bytes from 10.0.2.2: icmp_seq=2 time=10ms
64 bytes from 10.0.2.2: icmp_seq=3 time=10ms
64 bytes from 10.0.2.2: icmp_seq=4 time=10ms

--- 10.0.2.2 ping statistics ---
4 packets transmitted, 4 received, 0% packet loss
```

## ICMP Echo 封包格式

```rust
struct IcmpEcho {
    icmp_type: u8,      // 8 = Echo Request, 0 = Reply
    code: u8,            // 0
    checksum: u16,
    identifier: u16,     // 0x1234
    sequence: u16,       // 遞增序號
    data: [u8; 32],      // "abcdefghijklmnopqrstuvwabcdefghi"
}
```

## 使用 PingSocket

ping 使用專用的 `PingSocket` 而非一般 UDP socket：

```rust
use xv8_user_std::net::PingSocket;

let socket = PingSocket::open().unwrap();
socket.send(&request, &dest_ip.0)?;
let (len, _src_ip) = socket.recv(&mut buf)?;
```

`PingSocket` 自動處理 ICMP 封包的封裝/解封裝。

## 回應解析

```rust
if let Some(reply) = icmp::parse_echo_reply(&buf[..len]) {
    println!("{} bytes from {}: icmp_seq={} time={}ms",
        PING_DATA.len(), dest_ip, seq, rtt);
}
```

## 時間計算

```rust
let t0 = uptime();           // 發送時間
// ... 發送並等待 ...
let t1 = uptime();           // 收到時間
let rtt = (t1 - t0) * 10;   // 轉換為毫秒 (每 tick = 10ms)
```

## 封包發送間隔

```rust
// 等待回應超時
while waited < TIMEOUT_TICKS { ... }

// 發送間隔 (最後一個不等待)
if i + 1 < count {
    sleep(10);
}
```

## 統計計算

```rust
let sent = 4;
let recv = 4;
let loss = ((sent - recv) * 100) / sent;  // 0%
```

## 與網路的整合

- 使用 `PingSocket::open()` 建立 ICMP socket
- 使用 `socket.send()` 傳送 ICMP Echo Request
- 使用 `socket.recv()` 接收 ICMP Echo Reply
- 支援 QEMU 用戶模式 NAT 環境

## 限制

- 不支援自訂資料大小 (固定 32 bytes)
- 不支援 record route 選項
- 不支援 timestamp 選項
- 不支援 IPv6

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| `pingsocket failed` | 無法建立 ICMP socket |
| `send failed` | 發送失敗 |
| `seq=N timeout` | 等待回應超時 |

## 相關主題

- [[ICMP]]：ICMP 協定
- [[TCP]]：TCP 傳輸
- [[UDP]]：UDP 傳輸