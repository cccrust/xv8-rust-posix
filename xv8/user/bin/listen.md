# listen — UDP 監聽伺服器

listen 監聽 UDP 連接埠並顯示收到的訊息。

## 使用方式

```bash
listen <port>
```

## 實作

```rust
fn wait_for_dhcp(fd: Fd) {
    let gw = [10, 0, 2, 2];
    let payload = b"dhcp probe";
    for _ in 0..MAX_RETRIES {
        match send(fd, payload, &gw, 9999) {
            Ok(_) => return,
            Err(e) => {
                assert_eq!(e, SysError::NoEntry);
                let _ = sleep(5);
            }
        }
    }
    panic!("DHCP did not complete");
}

fn main(args: Args) {
    let port: u16 = args.get_str(1).unwrap().parse().unwrap_or_else(|_| exit(1));

    let fd = socket(0).unwrap_or_else(|_| exit(1));
    wait_for_dhcp(fd);

    let bind_fd = socket(port).unwrap_or_else(|_| exit(1));

    loop {
        let mut buf = [0u8; 512];
        let mut src_ip = [0u8; 4];
        let mut src_port = 0u16;

        match receive(bind_fd, &mut buf, &mut src_ip, &mut src_port) {
            Ok(n) => {
                let from = Ipv4Addr(src_ip);
                let msg = core::str::from_utf8(&buf[..n]).unwrap_or("<binary>");
                println!("{}:{} - {}", from, src_port, msg);
            }
            Err(_) => {
                let _ = sleep(1);
            }
        }
    }
}
```

## 流程

1. 解析連接埠參數
2. 建立 UDP socket
3. 等待 DHCP 完成（發送探測封包直到成功或超時）
4. 監聽指定連接埠
5. 無限迴圈接收並顯示訊息

## DHCP 等待

```rust
const MAX_RETRIES: usize = 100;

fn wait_for_dhcp(fd: Fd) {
    // 發送 dhcp probe 到閘道器
    // 如果收到 NoEntry 錯誤，重試
    // 成功後返回
}
```

這確保在接收訊息前網路已配置完成。

## 訊息顯示

```rust
let from = Ipv4Addr(src_ip);
let msg = core::str::from_utf8(&buf[..n]).unwrap_or("<binary>");
println!("{}:{} - {}", from, src_port, msg);
```

- 顯示來源 IP 和連接埠
- 嘗試解析為 UTF-8 文字
- 二進制資料顯示為 `<binary>`

## 錯誤處理

| 錯誤 | 處理 |
|------|------|
| `invalid port` | 輸出用法並退出 |
| `socket failed` | 輸出錯誤並退出 |
| `DHCP did not complete` | panic |
| `receive` timeout | sleep 1 後重試 |

## 範例

```bash
# 監聽 UDP 連接埠 12345
listen 12345

# 輸出範例
10.0.2.2:54321 - Hello from client
10.0.2.3:54322 - Another message
```

## 與 udp listen 的差異

| 功能 | listen | udp listen |
|------|--------|-------------|
| DHCP 等待 | 有 | 無 |
| 來源顯示 | IP:Port | 無 |
| 錯誤處理 | 嚴格 | 宽容 |

## 相關主題

- [[udp]]：UDP 工具
- [[dns]]：DNS 查詢
- [[TCP]]：TCP 監聽