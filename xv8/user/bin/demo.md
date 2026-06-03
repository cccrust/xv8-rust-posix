# demo — 系統功能展示

demo 是一個互動式展示程式，示範 xv8 核心的多程序管理、管線通訊和平行計算能力。

## 輸出範例

```
PID: 3  |  Uptime: 1234 ticks

[1] Process Management
    Forking 4 worker processes...
    Worker 1: PID 4
    Worker 2: PID 5
    Worker 3: PID 6
    Worker 4: PID 7
    All 4 workers exited.

[2] Pipe IPC
    Producer PID 8  ->  14 bytes: "Hello from child!"

[3] Parallel Timing (4 CPUs x 10 ticks)
    Serial estimate:  40 ticks
    Spawning workers...
    Parallel actual:  11 ticks
    Speedup:          ~3x  (ideal: 4x)

done
```

## 三大部分

### 1. 程序管理 (demo_process_management)

```rust
fn demo_process_management() {
    // fork NCPU 個子程序
    for slot in child_pids.iter_mut() {
        match fork().unwrap() {
            0 => exit(0),           // 子程序立即退出
            pid => *slot = pid,    // 父程序記錄 PID
        }
    }
    // 等待所有子程序退出
    for _ in 0..NCPU {
        wait(&mut status).expect("wait failed");
    }
}
```

驗證：核心能同時管理多個程序並正確配置 PID。

### 2. 管線 IPC (demo_pipe_ipc)

```rust
let (read_fd, write_fd) = pipe().unwrap();
// fork
// 子程序: 關閉讀端, 寫入訊息
// 父程序: 關閉寫端, 讀取訊息
```

驗證：管線緩衝、阻塞讀取、EOF 訊號傳遞。

### 3. 平行計時 (demo_parallel_timing)

```rust
// 同時 sleep SLEEP_TICKS
for _ in 0..NCPU {
    if fork() == 0 {
        sleep(SLEEP_TICKS);
        exit(0);
    }
}
// 等待所有子程序
// 測量總時間
```

理想加速比 = NCPU (4x)，實際接近此值表示核心正確調度多核心。

## 設定常數

| 常數 | 值 | 說明 |
|------|-----|------|
| `SLEEP_TICKS` | 10 | 每個 worker 睡眠時間 (100ms) |
| `NCPU` | 4 | worker 數量 (需與 QEMU -smp 一致) |

## 與核心的整合

- 使用 `fork()` 建立程序
- 使用 `wait()` 等待程序退出
- 使用 `pipe()` 建立程序間通訊
- 使用 `sleep()` 測試調度
- 使用 `uptime()` 測量時間

## 相關主題

- [[fork]]：程序建立
- [[Pipe]]：程序間通訊
- [[primes]]：平行計算