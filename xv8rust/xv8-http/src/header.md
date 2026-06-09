# Header — HTTP 標頭

`header.rs` 定義 HTTP 標頭（header）的資料結構，包括 `HeaderName`、`HeaderValue`、`HeaderMap`。

## HTTP 標頭概念

HTTP 標頭是鍵值對的集合，位於請求/回應的起始行（start line）之後。標頭提供請求/回應的中繼資料：

- **通用標頭**: Date、Connection、Cache-Control
- **請求標頭**: Host、User-Agent、Accept、Authorization、Cookie
- **回應標頭**: Server、Set-Cookie、WWW-Authenticate
- **實體標頭**: Content-Type、Content-Length、Content-Encoding

## HeaderMap 設計

`HeaderMap` 是針對 HTTP 標頭優化的映射結構：

- 不區分大小寫的鍵查詢（RFC 7230 規定標頭名稱不區分大小寫）
- 多值支援（同一標頭可出現多次，如 Set-Cookie、Accept）
- 高效迭代

## 相關文件

- [request.md](./request.md) — 請求結構
- [response.md](./response.md) — 回應結構
- [parse.md](./parse.md) — 標頭解析
