# TCP — 傳輸控制協定

TCP 是連接導向、可靠的傳輸層協定。

## 特性

- **連接導向**：三次握手順立連線
- **可靠**：確認、重傳機制
- **順序**：封包編號確保順序
- **流量控制**：滑動視窗
- **擁塞控制**：慢啟動、擁塞避免

## 狀態機

```
CLOSED → SYN_SENT → ESTABLISHED → FIN_WAIT_1 → FIN_WAIT_2
   │           │              │              │
   ▼           ▼              ▼              ▼
LISTEN ←───── SYN_RECV      CLOSE_WAIT    TIME_WAIT
                │              │              │
                ▼              ▼              ▼
             (並進入 ESTABLISHED)
```

## xv8 TCP 狀態

```rust
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
}
```

## TCP 標頭

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├───────────────────┬───────────────────┬───────────────────────┤
│   Source Port     │   Destination Port                    │
├───────────────────┴───────────────────┼───────────────────────┤
│                        Sequence Number                      │
├─────────────────────────────────────┼───────────────────────┤
│                    Acknowledgment Number                   │
├───────┬───────┬─────────────────────┼───────────────────────┤
│ Data  │       │                     │                       │
│ Offset│ Flags │      Window        │      Checksum         │
├───────┴───────┴─────────────────────┴───────────────────────┤
│                   Urgent Pointer                          │
├───────────────────────────────────────────────────────────┤
│                      Options (if Data Offset > 5)        │
└───────────────────────────────────────────────────────────┘
```

## xv8 TCP 標頭

```rust
#[repr(C, packed)]
pub struct TcpHeader {
    src_port: Be<u16>,
    dest_port: Be<u16>,
    seq_num: Be<u32>,
    ack_num: Be<u32>,
    off_flags: Be<u16>,   // 資料偏移 + 旗標
    window: Be<u16>,
    checksum: Be<u16>,
    urgent: Be<u16>,
}
```

### 旗標

```rust
const TCP_FIN: u8 = 1;
const TCP_SYN: u8 = 2;
const TCP_RST: u8 = 4;
const TCP_PSH: u8 = 8;
const TCP_ACK: u8 = 16;
```

## 三次握手

```
客戶端                              伺服器
   │                                   │
   │────────── SYN (seq=1000) ────────►│  1. 主動開啟
   │                                   │
   │◄──────── SYN+ACK (seq=2000,ack=1001)│  2. 被動開啟 + 確認
   │                                   │
   │────────── ACK (ack=2001) ─────────►│  3. 確認
   │                                   │
   ▼                                   ▼
```

## xv8 連線建立

### 客戶端（主動開啟）

```rust
pub fn connect(id: usize, remote_ip: Ipv4Addr, remote_port: u16) -> Result<(), NetError> {
    // 發送 SYN
    transmit_tcp(remote_ip, remote_port, local_port, seq, 0, TCP_SYN, &[])?;

    // 等待 SYN-ACK
    loop {
        let entry = table.entries[id].as_mut().ok_or(NetError::BadSocket)?;
        if matches!(entry.state, TcpState::Established) {
            return Ok(());
        }
        table = proc::sleep(Channel::Buffer(entry as *const _ as usize), table);
    }
}
```

### 伺服器（被動開啟）

```rust
// 收到 SYN
if has_syn && !has_ack {
    // 建立 child connection
    table.entries[child_id] = Some(child);
    // 發送 SYN-ACK
    transmit_tcp(src_ip, src_port, lport, 2000, seq.wrapping_add(1), TCP_SYN | TCP_ACK, &[]);
}
```

## 連線表

```rust
const NTCP: usize = 16;

static TCP_TABLE: SpinLock<TcpTable> = SpinLock::new(
    TcpTable {
        entries: [const { None }; NTCP],
        next_ephemeral: EPHEMERAL_PORT_START,
    },
    "tcp_table",
);
```

## 連線條目

```rust
pub struct TcpConnection {
    state: TcpState,
    local_port: u16,
    remote_ip: Ipv4Addr,
    remote_port: u16,
    send_seq: u32,
    recv_seq: u32,
    send_buf: Vec<u8>,
    recv_buf: Vec<u8>,
    recv_ready: bool,
    backlog: Vec<usize>,
}
```

## 資料傳輸

```rust
pub fn send(id: usize, data: &[u8]) -> Result<usize, NetError> {
    let len = data.len().min(TCP_MAX_SEG);  // 1460
    transmit_tcp(rip, rport, lport, seq, ack, TCP_PSH | TCP_ACK, &data[..len])?;
    entry.send_seq = entry.send_seq.wrapping_add(len as u32);
    Ok(len)
}
```

## 連線關閉

```rust
pub fn close(id: usize) {
    match state {
        TcpState::Established => {
            // 發送 FIN
            transmit_tcp(remote_ip, remote_port, local_port, seq, ack, TCP_FIN | TCP_ACK, &[]);
            entry.state = TcpState::FinWait1;
        }
        TcpState::CloseWait => {
            transmit_tcp(remote_ip, remote_port, local_port, seq, ack, TCP_FIN | TCP_ACK, &[]);
            entry.state = TcpState::LastAck;
        }
    }
}
```

## 最大段大小

```rust
pub const TCP_MAX_SEG: usize = 1460;  // 1500 - 20(IP) - 20(TCP)
```

## 滑動視窗

xv8 目前未完整實現滑動視窗，固定使用 65535 視窗大小。

## 計時器

xv8 目前未實現：
- 重傳計時器
- 持續計時器
- TIME_WAIT 計時器

## 限制

- 不支援部分可靠性功能
- 不支援明確擁塞通知（ECN）
- 有限連線數（16 個）

## 安全性考量

- TCP 序列號預測攻擊
- SYN Flood
- 連線劫持

## 相關主題

- [[IPv4]]：IP 層
- [[Socket]]：BSD Socket API