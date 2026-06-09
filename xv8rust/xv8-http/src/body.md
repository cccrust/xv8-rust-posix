# Body — HTTP 訊息主體

`body.rs` 定義 HTTP 請求/回應的主體（body）類型。HTTP 主體攜帶請求或回應的實際資料，在請求和回應訊息中位於標頭之後，由空行分隔。

## Body 類型

xv8-http 的 `Body` 型別支援三種承載形式：

- **Bytes**: 完整的位元組緩衝區，適合已知大小的主體
- **Stream**: 串流形式的主體，用於 chunked transfer encoding 或未知長度
- **Empty**: 無主體（如 GET 請求）

## Content-Length 與 Transfer-Encoding

HTTP/1.1 使用以下機制標示主體結束：

- **Content-Length**: 主體長度以位元組為單位，接收端據此知道讀取多少資料
- **Transfer-Encoding: chunked**: 主體以 chunk 形式傳送，每個 chunk 前有長度資訊，最後以 0-length chunk 結束

## 相關文件

- [request.md](./request.md) — HTTP 請求結構
- [response.md](./response.md) — HTTP 回應結構
- [parse.md](./parse.md) — HTTP 解析器
