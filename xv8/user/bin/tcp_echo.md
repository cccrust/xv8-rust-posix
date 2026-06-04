# tcp_echo — TCP Echo 伺服器

tcp_echo 監聽 TCP 連接，接受客戶端連線並回應收到的資料。

## 使用方式

```bash
tcp_echo <port>
```

## 實作

```rust
fn main(args: Args) {
    let port = args.get_str(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| exit_with_msg("invalid port number"));

    let fd = tcp_socket().expect("tcp_socket failed");
    tcp_bind(fd, port).expect("tcp_bind failed");
    tcp_listen(fd).expect("tcp_listen failed");

    let mut buf = [0u8; 4096];

    loop {
        let client = tcp_accept(fd).expect("tcp_accept failed");
        println!("accepted connection on fd={}", client.as_raw());

        let n = tcp_recv(client, &mut buf).expect("tcp_recv failed");
        if n > 0 {
            let _ = tcp_send(client, &buf[..n]);  // Echo back
        }

        close(client).expect("close failed");
    }
}
```

## TCP 伺服器流程

```
1. tcp_socket()     建立 TCP socket
2. tcp_bind(port)   綁定到指定連接埠
3. tcp_listen()     監聽連線
4. loop {
       tcp_accept()    接受客戶端連線
       tcp_recv()       接收資料
       tcp_send()       發送回應
       close()          關閉連線
   }
```

## Echo 行為

收到的任何 TCP 資料都會原封不動地回傳給客戶端：
- `n > 0`：有收到資料
- `n == 0`：連線關閉
- 回應大小等於收到的資料大小

## 連接處理

每個客戶端連線：
1. `tcp_accept()` 返回新的客戶端 socket fd
2. 從客戶端讀取資料
3. 將資料回顯給客戶端
4. 關閉客戶端連線

伺服器主 socket 保持監聽，可接受多個連線。

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| `invalid port number` | 連接埠號無效 |
| `tcp_socket failed` | 無法建立 socket |
| `tcp_bind failed` | 無法綁定連接埠 |
| `tcp_listen failed` | 無法監聽 |
| `tcp_accept failed` | 接受連線失敗 |

## 測試方式

```bash
# 終端 1：啟動伺服器
tcp_echo 7

# 終端 2：連線並測試
nc 10.0.2.15 7
hello        # 輸入
hello        # 收到回應
```

## 與 TCP 的整合

- 使用 `tcp_socket()` 建立 TCP socket
- 使用 `tcp_bind()` 綁定位址
- 使用 `tcp_listen()` 開始監聽
- 使用 `tcp_accept()` 接受連線
- 使用 `tcp_recv()` 接收資料
- 使用 `tcp_send()` 發送資料

## 限制

- 單執行緒，串列處理客戶端
- 無法同時處理多個客戶端
- 無最大連線數限制

## 相關主題

- [[TCP]]：TCP 協定
- [[listen]]：UDP 監聽