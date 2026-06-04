# date — 顯示或設定系統時間

`date` 用於顯示系統時間，可格式化輸出。

## 核心設計

```rust
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs() as i64;

unsafe {
    let mut tm: libc::tm = std::mem::zeroed();
    let t = now as libc::time_t;
    libc::localtime_r(&t, &mut tm);  // 轉換為本地時間結構
}
```

使用 `localtime_r` 或 `gmtime_r` 將 Unix timestamp 轉換為Broken-down time。

## Unix Time（Epoch Time）

```rust
// 自 1970-01-01 00:00:00 UTC 以來的秒數
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .as_secs() as i64;
```

Unix time 是：
- 連續的（沒有閏秒）
- 基於 UTC
- 可跨時區轉換

## 時間結構

```rust
struct tm {
    tm_sec: i32,    // 秒（0-60，可能有閏秒）
    tm_min: i32,    // 分（0-59）
    tm_hour: i32,   // 時（0-23）
    tm_mday: i32,   // 日（1-31）
    tm_mon: i32,    // 月（0-11，加 1 得到 1-12）
    tm_year: i32,    // 年（從 1900 年開始，加 1900 得到實際年份）
    tm_wday: i32,    // 星期幾（0=週日）
    tm_yday: i32,    // 一年中的第幾天（0-365）
    tm_isdst: i32,   // 夏令時旗標（正=啟用，0=未啟用，負=未知）
}
```

## 格式化輸出

```rust
fn emit_format(fmt: &str, y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) {
    let mut out = fmt.replace("%Y", &format!("{:04}", y));  // 4 位年份
    out = out.replace("%m", &format!("{:02}", m));  // 2 位月
    out = out.replace("%d", &format!("{:02}", d));  // 2 位日
    out = out.replace("%H", &format!("{:02}", h));  // 2 位時（24小時制）
    out = out.replace("%M", &format!("{:02}", min));
    out = out.replace("%S", &format!("{:02}", s));
    println!("{}", out);
}
```

常用格式字串：
- `%Y-%m-%d`：ISO 日期（2024-01-15）
- `%H:%M:%S`：時間（14:30:00）
- `%Y-%m-%d %H:%M:%S`：完整時間
- `%a %b %d %H:%M:%S %Y`：預設格式

## UTC vs 本地時間

```rust
if utc {
    libc::gmtime_r(&t, &mut tm);  // UTC
} else {
    libc::localtime_r(&t, &mut tm);  // 本地時間
}
```

- `date`：顯示本地時間
- `date -u`：顯示 UTC 時間

## 時區處理

`localtime_r` 使用系統的時區設定：
- Linux：環境變數 `TZ`
- 讀取 `/etc/localtime`

## 閏年演算法

```rust
fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
```

每四年一閏，百年不閏，四百年又閏。

## 閏秒處理

`tm_sec` 的範圍是 0-60，以支援閏秒。這在某些系統上可能導致時間異常。

## 典型用途

```bash
# 顯示目前時間
date

# 顯示 UTC 時間
date -u

# 格式化輸出
date "+%Y-%m-%d %H:%M:%S"

# 產生時間戳
date "+%s"

# 格式化檔案名（備份）
cp file.txt file_$(date +%Y%m%d).txt
```

## 與 time 命令的比較

- `date`：顯示/設定系統時間
- `time`：測量命令執行時間

## 設置時間

在 Linux 上（需要 root）：
```bash
date -s "2024-01-15 14:30:00"
```

## 夏令時（DST）

`tm_isdst` 指示是否在夏令時期間。這會影響 `localtime` 的結果。

## 跨平台差異

- Linux：使用 `localtime_r`
- macOS：類似的函式
- Windows：使用不同的 API

## 底層系統呼叫

- `time()`/`gettimeofday()`：獲取當前時間
- `localtime()`：轉換為本地時間
- `gmtime()`：轉換為 UTC

## 與其他工具的結合

```bash
# 測量命令執行時間
time ./command

# 日誌時間戳
logger "$(date +%Y-%m-%d\ %H:%M:%S) message"

# 檔案時間戳
touch -d "$(date)" filename
```

## 相關指令

- `cal`：顯示日曆
- `tzselect`：選擇時區
- `ntpdate`：網路時間同步
- `hwclock`：硬體時鐘