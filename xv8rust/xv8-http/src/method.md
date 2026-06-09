# Method — HTTP 請求方法

`method.rs` 定義 HTTP 請求方法（request method）列舉，對應 RFC 7231 與 RFC 5789 定義的標準方法。

## HTTP 請求方法

HTTP 方法指示請求的意圖（intent），是 RESTful API 的核心：

| 方法 | RFC | 語意 | 冪等 | 安全 |
|------|-----|------|------|------|
| GET | 7231 | 取得資源 | 是 | 是 |
| HEAD | 7231 | 僅取得標頭 | 是 | 是 |
| POST | 7231 | 建立資源或提交資料 | 否 | 否 |
| PUT | 7231 | 更新/取代資源 | 是 | 否 |
| DELETE | 7231 | 刪除資源 | 是 | 否 |
| PATCH | 5789 | 部分修改資源 | 否 | 否 |
| OPTIONS | 7231 | 查詢支援的方法 | 是 | 是 |
| TRACE | 7231 | 診斷回顯請求 | 是 | 是 |
| CONNECT | 7231 | 建立隧道（TLS） | 否 | 否 |

## 安全與冪等

- **安全方法**: 不應有副作用（GET、HEAD、OPTIONS、TRACE）
- **冪等方法**: 單次與多次請求效果相同（GET、HEAD、PUT、DELETE、OPTIONS、TRACE）

## 相關文件

- [lib.md](./lib.md) — HTTP 類型總覽
- [request.md](./request.md) — 請求結構
- [parse.md](./parse.md) — 請求解析
