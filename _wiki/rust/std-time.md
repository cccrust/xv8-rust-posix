# std::time

時間和計時器功能。

## Duration

時間段：

```rust
use std::time::Duration;

let dur = Duration::from_secs(5);         // 5 秒
let dur = Duration::from_millis(1500);     // 1500 毫秒
let dur = Duration::from_micros(1_000_000); // 1 秒（微秒）
let dur = Duration::from_nanos(1_000_000_000); // 1 秒（奈秒）
```

### 運算

```rust
let a = Duration::from_secs(5);
let b = Duration::from_secs(1);

let sum = a + b;     // 6 秒
let diff = a - b;    // 4 秒
let double = a * 2;  // 10 秒
let half = a / 2;    // 2 秒
```

### as_* 方法

```rust
let dur = Duration::from_secs(125);

assert_eq!(dur.as_secs(), 125);
assert_eq!(dur.as_millis(), 125_000);
assert_eq!(dur.as_micros(), 125_000_000);
assert_eq!(dur.as_nanos(), 125_000_000_000);
assert_eq!(dur.as_secs_f64(), 125.0);
```

## Instant

時間點（相對）：

```rust
use std::time::{Instant, Duration};

let start = Instant::now();

// 做些工作...
std::thread::sleep(Duration::from_millis(10));

let elapsed = start.elapsed();
println!("Elapsed: {:?}", elapsed);
println!("Millis: {}", elapsed.as_millis());
```

### timeout

```rust
use std::time::{Instant, Duration};

let deadline = Instant::now() + Duration::from_secs(5);

if Instant::now() < deadline {
    // 還有時間
}
```

## SystemTime

系統時間（絕對）：

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now();
println!("{:?}", now);

// 取得 Unix timestamp
let duration = now.duration_since(UNIX_EPOCH).unwrap();
println!("Unix time: {} seconds", duration.as_secs());
```

### 轉換

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let timestamp: u64 = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as u64;

// 從 timestamp 創建
let time = UNIX_EPOCH + Duration::from_secs(1234567890);
```

## UNIX_EPOCH

1970 年 1 月 1 日 00:00:00 UTC：

```rust
use std::time::UNIX_EPOCH;

let epoch = UNIX_EPOCH;
let now = SystemTime::now();

if now > epoch {
    if let Ok(dur) = now.duration_since(epoch) {
        println!("Seconds since epoch: {}", dur.as_secs());
    }
}
```

## 本專案使用

### traceroute 超時

```rust
use std::time::{Duration, Instant};

let deadline = Instant::now() + Duration::from_secs(3);
socket.set_read_timeout(Some(Duration::from_secs(3)))?;
```

### ping 計時

```rust
use std::time::{Duration, Instant};

let send_time = Instant::now();
socket.send_to(&packet, &addr)?;
let elapsed = send_time.elapsed();
```

### sleep

```rust
use std::thread;
use std::time::Duration;

thread::sleep(Duration::from_secs(1));
```

### NTP 時間同步

```rust
let t1 = Instant::now();          // 客戶端發送時間
// ... 網路傳輸 ...
let t2 = Instant::now();           // 伺服器接收時間
let t3 = Instant::now();           // 伺服器發送時間
// ... 網路傳輸 ...
let t4 = Instant::now();           // 客戶端接收時間

let offset = ((t2 - t1) - (t4 - t3)) / 2;
let delay = (t4 - t1) - (t3 - t2);
```

## UNIX timestamp

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let now = SystemTime::now();
let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
```

## 鬧鐘

```rust
use std::time::{Instant, Duration};

let wake_time = Instant::now() + Duration::from_secs(10);

loop {
    if Instant::now() >= wake_time {
        break;
    }
    std::thread::sleep(Duration::from_millis(100));
}
```

## 與作業系統的關係

- **Linux**：clock_gettime(CLOCK_REALTIME)
- **macOS**：gettimeofday
- **xv8**：透過 syscall

## 精度

| 方法 | 精度 |
|------|------|
| `Duration::from_secs` | 秒 |
| `Duration::from_millis` | 毫秒 |
| `Duration::from_micros` | 微秒 |
| `Duration::from_nanos` | 奈秒 |

實際精度取決於硬體時脈。

## 日期時間

std::time 只處理時間點和時段，不處理日曆日期（如「2024年1月1日」）。需要日曆功能時，使用 chrono 之類的外部 crate。

## 相關模組

- `std::thread::sleep`：執行緒睡眠
- `std::net`：網路超時
- `std::process`：程序計時