# Status — HTTP 狀態碼

`status.rs` 定義 HTTP 狀態碼（StatusCode）列舉與其對應的原因短語（reason phrase）。

## 常見狀態碼

### 2xx 成功

| 碼 | 短語 | 語意 |
|-----|------|------|
| 200 | OK | 請求成功 |
| 201 | Created | 資源已建立（POST/PUT） |
| 204 | No Content | 成功但無主體（DELETE） |

### 3xx 重定向

| 碼 | 短語 | 語意 |
|-----|------|------|
| 301 | Moved Permanently | 永久重定向 |
| 302 | Found | 暫時重定向 |
| 304 | Not Modified | 快取未過期 |

### 4xx 用戶端錯誤

| 碼 | 短語 | 語意 |
|-----|------|------|
| 400 | Bad Request | 請求格式錯誤 |
| 401 | Unauthorized | 需要認證 |
| 403 | Forbidden | 無權限（認證後） |
| 404 | Not Found | 資源不存在 |
| 405 | Method Not Allowed | 方法不支援 |
| 429 | Too Many Requests | 速率限制 |

### 5xx 伺服器錯誤

| 碼 | 短語 | 語意 |
|-----|------|------|
| 500 | Internal Server Error | 伺服器內部錯誤 |
| 502 | Bad Gateway | 閘道器無效回應 |
| 503 | Service Unavailable | 暫時無法服務 |
| 504 | Gateway Timeout | 閘道器逾時 |

## 相關文件

- [response.md](./response.md) — 回應結構
- [lib.md](./lib.md) — HTTP 類型總覽
