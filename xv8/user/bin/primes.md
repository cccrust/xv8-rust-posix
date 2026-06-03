# primes — 質數計算 (平行版本)

primes 計算 [2, LIMIT) 範圍內的質數，比較平行和串列執行的效能。

## 使用方式

```bash
primes
```

## 輸出範例

```
Range: [2, 30000000)  |  Workers: 4

[Parallel]
  Worker 0: [2, 7500000)
  Worker 1: [7500000, 15000000)
  Worker 2: [15000000, 22500000)
  Worker 3: [22500000, 30000000)
  Found 1908883 primes  |  Time: 123 ticks

[Serial]
  Found 1908883 primes  |  Time: 456 ticks

Speedup: ~3x  (ideal: 4x)
```

## 演算法

### 質數判斷 (試除法)

```rust
fn is_prime(n: usize) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n.is_multiple_of(2) { return false; }

    let mut d = 3;
    while d * d <= n {
        if n.is_multiple_of(d) { return false; }
        d += 2;
    }
    true
}
```

優化：
- 排除偶數
- 只除到 √n

### 範圍劃分

```rust
let range_size = LIMIT / NCPU;  // 每個 worker 的範圍

let worker_start = i * range_size;
let worker_end = if i + 1 == NCPU { LIMIT } else { (i + 1) * range_size };
```

## 管線架構

```
         fork
           │
    ┌──────┼──────┬───────┐
    │      │      │       │
pipe0   pipe1   pipe2   pipe3
    │      │      │       │
 Worker0 Worker1 Worker2 Worker3
    │      │      │       │
    └──────┴──────┴───────┘
              │
         close write
              │
         read from each pipe
              │
         sum results
```

## 子程序工作流程

```rust
if fork() == 0 {
    // 關閉不需要的 pipe 端
    for (j, &(read_fd, write_fd)) in pipes.iter().enumerate() {
        close(read_fd);
        if j != i {
            close(write_fd);
        }
    }

    // 計算質數
    let count = count_primes(worker_start, worker_end) as u64;

    // 寫入結果到 pipe
    pipes[i].1.write_all(&count.to_le_bytes())?;
    close(pipes[i].1);

    exit(0);
}
```

## 結果收集

```rust
// 關閉所有寫端 (這樣讀端才會 EOF)
for &(_, write_fd) in pipes.iter() {
    close(write_fd).expect("close failed");
}

// 讀取每個 pipe 的結果
let mut par_total = 0;
for &mut (mut read_fd, _) in pipes.iter_mut() {
    let mut buf = [0u8; 8];
    read_fd.read_exact(&mut buf)?;
    close(read_fd)?;
    par_total += u64::from_le_bytes(buf) as usize;
}
```

## 驗證

```rust
if par_total != ser_total {
    eprintln!("ERROR: parallel ({}) and serial ({}) counts disagree!", par_total, ser_total);
    exit(1);
}
```

確保平行結果與串列結果一致。

## 效能測量

```rust
let par_start = uptime();
// fork 並等待 NCPU 個 worker
let par_elapsed = uptime().saturating_sub(par_start);

let ser_start = uptime();
let ser_total = count_primes(2, LIMIT);
let ser_elapsed = uptime().saturating_sub(ser_start);

let speedup = ser_elapsed / par_elapsed;  // 理想為 NCPU
```

## 設定常數

| 常數 | 值 | 說明 |
|------|-----|------|
| `LIMIT` | 30,000,000 | 質數搜尋上限 |
| `NCPU` | 4 | worker 數量 |

## 與核心的整合

- 使用 `fork()` 建立 worker 程序
- 使用 `pipe()` 建立程序間通訊
- 使用 `sleep()` 測量調度
- 使用 `wait()` 等待 worker 結束
- 使用 `uptime()` 測量時間

## 限制

- 靜態 worker 數量（需與 -smp 匹配）
- 質數計算使用簡單試除法（可進一步優化）

## 相關主題

- [[demo]]：平行計算展示
- [[fork]]：程序建立
- [[Pipe]]：程序間通訊