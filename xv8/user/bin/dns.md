# dns — DNS 查詢工具

dns 傳送 DNS A 記錄查詢並解析網域名稱為 IPv4 位址。

## 使用方式

```bash
dns <server> <domain>
```

## 範例

```bash
dns 10.0.2.3 google.com
```

## DNS 查詢格式

```
+-------------+-------------+---------------------------+
| Header (12) | Question    | Answer (optional)         |
+-------------+-------------+---------------------------+
| ID | Flags | QNAME | QTYPE | A record (if found)     |
+-------------+-------------+---------------------------+
```

### 標頭格式

| 位移 | 大小 | 說明 |
|------|------|------|
| 0 | 2 | ID (匹配請求/回應) |
| 2 | 2 | Flags (0x0100 = 標準查詢) |
| 4 | 2 | QDCOUNT (問題數) |
| 6 | 2 | ANCOUNT (答案數) |

### 問題格式

```rust
// 查詢: "example.com"
// 編碼為: 7 "example" 3 "com" 0
buf[pos] = label.len() as u8;           // 標籤長度
buf[pos+1..].copy_from_slice(label);    // 標籤內容
pos += 1 + label.len();
buf[pos] = 0;                           // 結尾
```

## 名稱解壓縮

DNS 回應使用指標壓縮，一個位元組可指向先前位置：

```rust
if len & 0xc0 == 0xc0 {
    // 指標: 11xxxxxx xxxxxxxx
    let ptr = ((len & 0x3f) << 8) | data[offset + 1];
    offset = ptr;  // 跳轉
}
```

## 答案解析

```rust
fn parse_ip_from_response(data: &[u8], query_id: u16) -> Result<[[u8; 4]; 16]> {
    // 檢查 ID 匹配
    // 檢查 flags (0x000f = 錯誤碼)
    // 解析 A 記錄 (rtype=1, rdlength=4)
    // 回傳 IPv4 位址陣列
}
```

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| `response too short` | 回應資料不完整 |
| `ID mismatch` | ID 不匹配 |
| `DNS error` | DNS 伺服器錯誤 |
| `no A records found` | 無 A 記錄 |
| `receive timeout` | 查詢超時 |

## 超時機制

```rust
const TIMEOUT_TICKS: usize = 50;  // 500ms

fn receive_timeout(fd, buf, ticks) {
    let step = 5;
    let mut waited = 0;
    while waited < ticks {
        if let Ok(n) = receive(fd, buf, ...) {
            return Ok(n);
        }
        sleep(step)?;
        waited += step;
    }
    Err(SysError::NoEntry)
}
```

## 與網路的整合

- 使用 `socket(0)` 建立 UDP 通訊端
- 使用 `send()` 傳送 DNS 查詢
- 使用 `receive_timeout()` 接收回應

## 限制

- 只支援 A 記錄查詢 (TYPE_A = 1)
- 只支援 IPv4 (CLASS_IN = 1)
- 不支援 DNSSEC
- 不支援其他記錄類型 (AAAA, MX, etc.)

## 相關主題

- [[UDP]]：UDP 傳輸
- [[Network-Stack]]：網路協定堆疊