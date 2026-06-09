# Lib — HTTP 類型函式庫

`lib.rs` 是 xv8-http crate 的根模組，提供 `#![no_std]` 相容的 HTTP/1.1 類型函式庫。

## 設計目標

xv8-http 提供 HTTP 協定所需的資料類型與解析功能，但刻意保持無 I/O、無網路依賴——它不知道如何透過 socket 發送或接收 HTTP 訊息，只處理 HTTP 訊息的結構化表示。

## 支援的類型

- **Method**: HTTP 請求方法（GET、POST、PUT、DELETE 等）
- **Uri**: URI 解析（scheme、authority、path、query）
- **Status**: HTTP 狀態碼（200 OK、404 Not Found 等）
- **HeaderMap**: 標頭集合
- **Request/Response**: 完整的 HTTP 請求和回應結構
- **Body**: 主體承載

## 相關文件

- [method.md](./method.md) — HTTP 方法
- [status.md](./status.md) — HTTP 狀態碼
- [uri.md](./uri.md) — URI 解析
- [request.md](./request.md) — 請求結構
- [response.md](./response.md) — 回應結構
