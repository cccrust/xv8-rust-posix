# Time — 命令執行時間測量

`time_`（因 Rust 關鍵字衝突更名）測量指定命令的執行時間。輸出三項計時：實際時間（wall clock）、使用者 CPU 時間（user）、系統 CPU 時間（sys）。實際時間可能大於 CPU 時間（I/O 等待），小於 CPU 時間（多核心）。Shell 內建 `time` 關鍵字與獨立 `/usr/bin/time` 行為略有不同。

## 相關文件

- [date.md](./date.md) — 日期時間顯示
- [sleep.md](./sleep.md) — 延遲執行
