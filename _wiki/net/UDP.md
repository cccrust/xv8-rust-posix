# UDP — 用戶資料報協定

UDP 是無連接導向的傳輸層協定，適合即時性應用。

## 特性

- **無連接**：不需要建立連線
- **不可靠**：不保證交付、順序或重複防護
- **輕量**：僅 8 位元組標頭

## 與 TCP 的比較

| 特性 | UDP | TCP |
|------|-----|-----|
| 連接導向 | 否 | 是 |
| 可靠性 | 無 | 有 |
| 流量控制 | 無 | 有 |
| 擁塞控制 | 無 | 有 |
| 標頭大小 | 8 位元組 | 20 位元組 |
| 速度 | 快 | 較慢 |

## UDP 標頭

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├───────────────────┬───────────────────┬───────────────────────┤
│     Source Port    │   Destination Port    │
├───────────────────┴───────────────────┴───────────────────────┤
│         Length          │        Checksum                 │
├───────────────────────┴──────────────────────────────────┤
│                         Data                               │
└───────────────────────────────────────────────────────────┘
```

## xv8 實作

### UDP 標頭

```rust
#[repr(C, packed)]
pub struct UdpHeader {
    src_port: Be<u16>,
    dest_port: Be<u16>,
    len: Be<u16>,
    sum: Be<u16>,
}
```

### 偽標頭

UDP 校驗和需要偽標頭（用於驗證）：
```rust
struct UdpPseudoHeader {
    src_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    zero: u8,
    proto: u8,           // 17 for UDP
    udp_len: Be<u16>,
}
```

## 通訊端管理

```rust
const NSOCKET: usize = 16;

static SOCKET_TABLE: SpinLock<SocketTable> = SpinLock::new(
    SocketTable {
        entries: [const { None }; NSOCKET],
        next_ephemeral: EPHEMERAL_PORT_START,
    },
    "sockets",
);
```

### 通訊端條目

```rust
struct SocketEntry {
    bound_ip: Ipv4Addr,       // 綁定的 IP (0.0.0.0 = 任意)
    bound_port: u16,         // 綁定的連接埠
    bound_interface: Option<InterfaceId>,
    receive_queue: [Option<ReceiveEntry>; MAX_RECV_QUEUE_DEPTH],
}

struct ReceiveEntry {
    src_ip: Ipv4Addr,
    src_port: u16,
    payload: Box<[u8]>,
}
```

## 通訊端操作

### 開啟

```rust
pub fn open(
    ip: Ipv4Addr,
    port: u16,
    interface: Option<InterfaceId>,
) -> Result<usize, NetError> {
    // 動態分配連接埠
    let bind_port = if port == 0 {
        table.next_ephemeral()
    } else {
        port
    };

    let Some(id) = table.entries.iter().position(|e| e.is_none()) else {
        err!(NetError::TableFull)
    };
}
```

### 傳送

```rust
pub fn send(socket_id: usize, dest_ip: Ipv4Addr, dest_port: u16, buf: &[u8]) -> Result<(), NetError> {
    let bound_port = SOCKET_TABLE.lock().entries[socket_id].as_ref().unwrap().bound_port;
    log!(transmit_udp(dest_ip, dest_port, bound_port, buf))
}
```

### 接收

```rust
pub fn receive(socket_id: usize) -> Result<(Ipv4Addr, u16, Box<[u8]>), NetError> {
    let entry = loop {
        let socket = table.entries[socket_id].as_mut().unwrap();
        if let Some(entry) = socket.dequeue() {
            break entry;
        }
        table = proc::sleep(Channel::Buffer(socket as *const _ as usize), table);
    };
    Ok((entry.src_ip, entry.src_port, entry.payload))
}
```

## 連接埠範圍

| 範圍 | 用途 |
|------|------|
| 0-1023 | 熟知連接埠 |
| 1024-49151 | 已登記連接埠 |
| 49152-65535 | 動態/暫時連接埠 |

xv8 動態連接埠：`49152` 起

##廣播支援

UDP 可傳送到廣播位址：

```rust
let dest_ip = Ipv4Addr::BROADCAST;  // 255.255.255.255
```

## 應用場景

- **DNS**：查詢/回應
- **DHCP**：IP 配置
- **NTP**：時間同步
- **RTP**：即時影音

## 限制

- 單一封包大小上限：取決於 IP 分片，通常 < 64KB
- 傳輸可靠性需由應用層處理

## 與 xv8 的整合

UDP 是 DNS 和 DHCP 的傳輸層基礎：
- DHCP 使用連接埠 67/68
- DNS 使用連接埠 53

## 安全性考量

- UDP flood：攻擊者傳送大量 UDP 封包
- 反射攻擊：利用 UDP 放大攻擊
- 連接埠掃描