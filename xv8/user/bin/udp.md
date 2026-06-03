# udp — UDP 發送/接收工具

udp 提供 UDP 訊息的發送和接收功能。

## 使用方式

```bash
# 監聽模式
udp listen <port>

# 發送模式
udp send <address> <port> [message]
```

## 監聽模式

```rust
let fd = socket(port).expect("socket failed");

loop {
    let rc_len = receive(fd, &mut buf, &mut src_ip, &mut src_port)?;
    println!("[{}:{}]: {}",
        Ipv4Addr(src_ip),
        src_port,
        str::from_utf8(&buf[..rc_len]).unwrap_or("<invalid utf-8>")
    );
}
```

行為：
1. 建立 UDP socket 監聽指定連接埠
2. 無限迴圈接收 UDP 封包
3. 顯示來源位址、連接埠和訊息

## 發送模式（單一訊息）

```rust
let fd = socket(0).expect("socket failed");
send(fd, message.as_bytes(), &dest_ip.0, dest_port)?;
```

行為：
1. 建立 UDP socket（連接埠 0 = 臨時連接埠）
2. 傳送指定訊息到目標位址

## 發送模式（ stdin）

```rust
let fd = socket(0).expect("socket failed");
loop {
    let len = read(Fd::STDIN, &mut buf)?;
    if len == 0 { break; }
    send(fd, buf[..len].strip_suffix(b"\n").unwrap_or(&buf[..len]), &dest_ip.0, dest_port)?;
}
```

行為：
1. 從標準輸入讀取
2. 每次讀取後立即發送
3. 移除結尾換行符
4. EOF 時結束

## 範例

```bash
# 監聽 UDP 連接
udp listen 12345

# 發送單一訊息
udp send 10.0.2.2 12345 "Hello"

# 從檔案發送
cat message.txt | udp send 10.0.2.2 12345
```

## UDP 特性

| 特性 | 說明 |
|------|------|
| 無連線導向 | 不需要建立連線 |
| 無可靠性 | 可能丟失或重複封包 |
| 無順序性 | 封包可能亂序抵達 |
| 較低延遲 | 比 TCP 開銷小 |

## 緩衝區大小

- 接收緩衝區：1024 bytes
- 發送緩衝區：取決於網路堆疊

## 與網路的整合

- 使用 `socket(port)` 建立 UDP socket
- 使用 `receive()` 接收 UDP 封包
- 使用 `send()` 發送 UDP 封包
- 使用 `read()` 從 stdin 讀取

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| `socket failed` | 無法建立 socket |
| `receive failed` | 接收失敗 |
| `send failed` | 發送失敗 |

## 限制

- 訊息必須為 UTF-8 可解析
- 二進制資料顯示為 `<invalid utf-8>`

## 相關主題

- [[UDP]]：UDP 協定
- [[dns]]：DNS 查詢
- [[listen]]：UDP 監聽伺服器