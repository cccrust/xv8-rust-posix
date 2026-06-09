# Time — 時間操作

`time.rs` 實作 `std::time` 模組，提供時間相關的函式與型別。

## 型別

- **Duration**: 時間間隔（秒 + 奈秒）
- **Instant**: 單調遞增的時間點（用於測量間隔）
- **SystemTime**: 系統時鐘時間（用於取得目前日期時間）

## xv8 的適應

xv8 的時間功能映射到核心系統呼叫：

- `time()`: 取得從 Unix epoch 到目前的秒數
- `clock_gettime(CLOCK_MONOTONIC)`: 取得單調時鐘（Instant）
- `clock_gettime(CLOCK_REALTIME)`: 取得實際時間（SystemTime）
- `nanosleep()`: 精確休眠

xv8 的時鐘中斷處理器（timer interrupt）以固定頻率（如 100 Hz）觸發，維護核心層級的時間計數。使用者空間透過上述系統呼叫取得時間資訊。

## 相關文件

- [rng.md](../../kernel/src/rng.md) — 時間熵源
- [timerfd.md](../../kernel/src/timerfd.md) — 計時器 fd
- [uptime.md](../../user/bin/uptime.md) — 系統運行時間
