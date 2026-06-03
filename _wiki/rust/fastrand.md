# fastrand

快速偽隨機數生成器，net/ 工具使用。

## 專案使用

```toml
[dependencies]
fastrand = "2"
```

## 基本用法

```rust
use fastrand::Rng;

let mut rng = fastrand::Rng::new();
let n: u32 = rng.u32(..);        // [0, u32)
let n: u32 = rng.u32(1..=6);     // [1, 6]
let b: bool = rng.bool();         // true 或 false
let c: char = rng.char(..);      // 任意字元
```

## 數值生成

```rust
let mut rng = fastrand::Rng::new();

// 各種範圍
rng.u8(..);      // 0-255
rng.u16(..);     // 0-65535
rng.u32(..);     // 0-2^32-1
rng.u64(..);     // 0-2^64-1
rng.usize(..);   // 平臺相關

// 有界範圍
let n = rng.u32(0..100);  // 0-99

// 浮點數
let f: f64 = rng.f64();   // [0.0, 1.0)
```

## 字節

```rust
let mut rng = fastrand::Rng::new();
let bytes: [u8; 16] = rng.array();

let mut vec = vec![0u8; 32];
rng.fill(&mut vec);
```

## 字串

```rust
let mut rng = fastrand::Rng::new();

// 隨機 ID
let id: String = rng.alphanumeric().take(16).collect();
let id: String = rng.hex(take(16)).collect();
```

## 選擇

```rust
let mut rng = fastrand::Rng::new();

let items = vec!["a", "b", "c"];
let choice = rng.choice(&items).unwrap();
```

## shuffle

洗牌：

```rust
let mut rng = fastrand::Rng::new();
let mut deck: Vec<i32> = (1..=52).collect();

rng.shuffle(&mut deck);
```

## 種子

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let seed = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos() as u64;

let mut rng = fastrand::Rng::with_seed(seed);
```

## 執行緒區域 RNG

```rust
use fastrand::Rng;

let n = Rng::with_seed(fastrand::seed());
```

## net/ 中的使用

### DNS 隨機選擇

```rust
let mut rng = fastrand::Rng::new();
let server = servers[rng.usize(0..servers.len())];
```

### 網路埠

```rust
let port: u16 = rng.u16(49152..65535);
```

### TCP 序號

```rust
let seq: u32 = rng.u32(..);
```

### 阻斷服務防護

```rust
let token: u64 = rng.u64(..);
```

## 與 rand crate 的比較

| 特性 | fastrand | rand |
|------|----------|------|
| 效能 | 快 | 較慢 |
| API | 簡單 | 複雜 |
| 依賴 | 少 | 多 |
| 同步 | 無 | 可選 |

## 安全性

`fastrand` 不是密碼學安全的安全隨機數生成器，不應用於安全相關場景。

## 演算法

fastrand 使用線性同餘生成器（LCG）或其他快速演算法，適合遊戲、模擬等場景。

## 效能優化

```rust
let mut rng = fastrand::Rng::new();

// 批量生成
let nums: Vec<u32> = (0..1000).map(|_| rng.u32(..)).collect();

// 使用 fill
let mut buf = [0u8; 1024];
rng.fill(&mut buf);
```

## 本專案依賴

```toml
# net/libnet/Cargo.toml
[dependencies]
fastrand = "2"

# net/tools/Cargo.toml
[dependencies]
fastrand = "2"
```

## 相關模組

- `std::time`：時間用作種子
- `rand`：更完整的隨機庫