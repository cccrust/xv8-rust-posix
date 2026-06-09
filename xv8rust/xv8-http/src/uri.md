# URI — 統一資源識別符

`uri.rs` 實作 URI（Uniform Resource Identifier）的解析與資料結構，遵循 RFC 3986。

## URI 結構

```
  scheme    authority         path      query   fragment
   |          |                |          |        |
  https://example.com:443/path/to/page?key=val#section
          |         |
       host       port
```

## URI 組成

| 組件 | 說明 | 範例 |
|------|------|------|
| Scheme | 協定類型 | `http`、`https`、`ftp` |
| Authority | 權威資訊（userinfo@host:port） | `user@example.com:8080` |
| Path | 資源路徑 | `/api/v1/users` |
| Query | 查詢參數 | `?page=1&limit=10` |
| Fragment | 片段識別（客戶端處理） | `#section-2` |

## 百分比編碼

URI 中保留字元（如 `:`, `/`, `?`, `#`, `[`, `]`, `@`, `!`, `$`, `&`, `'`, `(`, `)`, `*`, `+`, `,`, `;`, `=`, `%`）需百分比編碼（percent-encoding）。xv8-http 的 `Uri` 支援解碼。

## 相關文件

- [request.md](./request.md) — 請求 URI
- [parse.md](./parse.md) — URI 解析
- [lib.md](./lib.md) — HTTP 類型總覽
