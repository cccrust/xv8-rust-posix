# Time — Tokio 時間相容層

`time.rs` 提供與 `tokio::time` 相容的非同步時間操作。

## 實作的功能

- **sleep(duration)**: 非同步等待一段時間（使用 `TimerEntry` 註冊到 reactor）
- **timeout(duration, future)**: 為 Future 設定超時，逾時回傳 `Elapsed` 錯誤
- **interval(duration)**: 建立固定間隔的定時器流

## 實作機制

tokio 的 time 模組透過內部計時器輪（timer wheel）管理睡眠任務。xv8 版本使用 `nanosleep` 系統呼叫與 reactor 的喚醒機制組合。

## 相關文件

- [time.md](../../xv8-user-std/src/time.md) — xv8 時間模組
- [timerfd.md](../../kernel/src/timerfd.md) — 計時器 fd
- [lib.md](./lib.md) — Tokio 相容層總覽
