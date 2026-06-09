# Col — 過濾反向換行字元

`col` 過濾處理反向換行（reverse line feed）字元，通常用於處理 nroff/troff 的格式化輸出。反向換行（ESC-7 或 SO）在終端機中游標回退到上一行，`col` 將其轉換為正向換行以適合純文字輸出。支援 `-b`（忽略退格）、`-x`（展開空白字元而不是退格）。

## 相關文件

- [expand.md](./expand.md) — Tab 展開
- [fmt.md](./fmt.md) — 文字格式化
