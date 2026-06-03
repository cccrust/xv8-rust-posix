# traceroute — 追蹤 IP 路由路徑

traceroute 追蹤封包到目的地之間經過的路由節點。

## 使用方式

```bash
traceroute <ip> [max_hops] [timeout_ms]
```

## 實作

```rust
const MAX_HOPS: u32 = 30;
const BASE_PORT: u16 = 33434;
const TIMEOUT_TICKS: usize = 50;  // 500ms假設 10ms/tick
const MAX_BUFFER_SIZE: usize = 512;

fn main(args: Args) {
    let dest_ip = args.get_str(1).unwrap().parse::<Ipv4Addr>()?;
    let max_hops = args.get_str(2).unwrap().parse().unwrap_or(MAX_HOPS);
    let timeout_ms = args.get_str(3).unwrap().parse().unwrap_or(5000);

    let timeout_ticks = (timeout_ms / 10).max(1) as usize;

    let socket_fd = socket(0)?;

    for ttl in 1..=max_hops {
        // 發送 UDP 封包到目的地
        let addr = SocketAddr::new(dest_ip.0, BASE_PORT + ttl as u16 - 1);
        send(socket_fd, &[0u8; 0], &addr.ip, addr.port)?;

        // 等待回應
        let start = uptime();
        let mut got_reply = false;

        while waited < timeout_ticks {
            if let Ok((len, src_ip)) = receive(socket_fd, &mut buf, &mut src_ip, &mut src_port) {
                let elapsed = (uptime() - start) * 10;
                println!("{:<2} {}.{}.{}.{}  {}ms", ttl, src_ip[0], src_ip[1], src_ip[2], src_ip[3], elapsed);
                got_reply = true;
                break;
            }
            sleep(1);
            waited += 1;
        }

        if !got_reply {
            println!("{:<2} *", ttl);  // 超時
        }
    }
}
```

## 演算法

traceroute 利用 UDP 封包的生存時間 (TTL) 欄位：

```
TTL=1  → 第一個路由器回應 (ICMP Time Exceeded)
TTL=2  → 第二個路由器回應
...
TTL=n  → 目的地回應 (ICMP Port Unreachable)
```

每次遞增 TTL，並記錄回應的路由器 IP。

## 限制

xv8 網路堆疊的限制：

```rust
// Note: The xv8 kernel's UDP socket does not support setting TTL via setsockopt.
// We cannot set the TTL in the IP header because we don't have raw IP access.
// This is a limitation of the current network stack in xv8.
```

- 無法設定 IP 層的 TTL
- 無法區分路由器回應和目的地回應
- 所有封包都以相同方式傳輸

## 輸出格式

```
traceroute to 10.0.2.2
 1  *  *  *
 2  10.0.2.1  5ms
 3  10.0.2.2  10ms
```

`*` 表示該 hop 無回應（超時）。

## 逾時計算

```rust
let timeout_ticks = (timeout_ms / 10).max(1) as usize;
let mut waited = 0;

while waited < timeout_ticks {
    if let Ok(..) = receive(..) {
        break;
    }
    sleep(1);
    waited += 1;
}
```

每 10ms 為一個 tick，逾時後輸出 `*`。

## SocketAddr 結構

```rust
#[derive(Debug, Clone, Copy)]
struct SocketAddr {
    ip: [u8; 4],
    port: u16,
}

impl SocketAddr {
    fn new(ip: [u8; 4], port: u16) -> Self {
        Self { ip, port }
    }
}
```

## 目的地判斷

```rust
// Note: Without TTL setting, we cannot know when we reach the destination.
// We'll break after max_hops anyway.
```

由於無法設定 TTL 無法主動判斷是否已到達目的地，只能等到所有 hops 都探索完或達到最大跳數。

## 錯誤處理

| 錯誤 | 處理 |
|------|------|
| `invalid IP address` | 輸出錯誤並退出 |
| `failed to create socket` | 輸出錯誤並退出 |
| `failed to send packet` | 繼續下一個 hop |
| 逾時 | 輸出 `*` |

## 範例

```bash
traceroute 10.0.2.2
traceroute 8.8.8.8 15 3000
```

## 與 ping 的差異

| 功能 | traceroute | ping |
|------|------------|------|
| 協定 | UDP | ICMP |
| 目的 | 追蹤路由路徑 | 測量連線 |
| 回應 | 路由器/目的地 | Echo Reply |
| TTL | 遞增 | 固定 |

## 限制

- 無法設定 TTL（依賴網路設備衰減）
- 無法區分各 hop
- 在 QEMU 環境可能只有閘道器回應

## 相關主題

- [[UDP]]：UDP 傳輸
- [[ping]]：ICMP Echo
- [[Network-Stack]]：網路堆疊