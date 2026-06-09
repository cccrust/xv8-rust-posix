# Csplit — 依上下文分割檔案

`csplit`（context split）根據上下文行（pattern）而非行數分割檔案。相對於 `split` 按大小分割，`csplit` 在每次匹配指定模式時建立新輸出檔案。支援正則表達式 pattern，可指定分割重複次數。用於將結構化檔案（如郵件匣、程式碼檔案）分割為邏輯區塊。

## 相關文件

- [split.md](./split.md) — 按大小分割
- [sed.md](./sed.md) — 串流編輯
