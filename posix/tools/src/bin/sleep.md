# sleep — 暫停執行

`sleep` 讓 shell 暫停執行指定的時間。

## 核心設計

```rust
fn main() {
    let total_secs: f64 = args[1..].iter()
        .filter_map(|s| s.parse::<f64>().ok())
        .sum();

    let secs = total_secs as u64;
    let nanos = ((total_secs - secs as f64) * 1_000_000_000.0) as u32;

    thread::sleep(Duration::new(secs, nanos));
}
```

`sleep` 解析參數為秒數（支援小數），然後呼叫執行緒睡眠。

## 時間參數

`sleep` 支援多種時間單位：

```bash
sleep 1        # 1 秒
sleep 0.5      # 0.5 秒
sleep 60        # 1 分鐘
sleep 1m        # 1 分鐘（某些版本）
sleep 1h        # 1 小時
sleep 1d        # 1 天
```

## 秒數解析

```rust
let total_secs: f64 = args[1..].iter()
    .filter_map(|s| s.parse::<f64>().ok())
    .sum();
```

多個參數會被加總：
```bash
sleep 1 2 3  # 等於 sleep 6
```

## Duration 結構

```rust
let secs = total_secs as u64;
let nanos = ((total_secs - secs as f64) * 1_000_000_000.0) as u32;

thread::sleep(Duration::new(secs, nanos));
```

- `Duration::new(secs, nanos)`：建立指定的時間長度
- 1 秒 = 1,000,000,000 奈秒

## 典型用途

### 延遲執行
```bash
echo "Starting in 5 seconds..."
sleep 5
./script.sh
```

### 迴圈中的延遲
```bash
while true; do
    ./check.sh
    sleep 60
done
```

### 倒計時
```bash
for i in $(seq 5 -1 1); do
    echo "T-minus $i"
    sleep 1
done
```

## 與 time 的比較

- `sleep`：讓程序暫停
- `time`：測量命令執行時間

## 腳本中的用途

```bash
# 重試迴圈
until curl -s http://example.com > /dev/null; do
    echo "Retrying in 10 seconds..."
    sleep 10
done
```

## 精度

`sleep` 的精度取決於：
- 作業系統的計時器解析度
- 系統負載

在 Linux 上，通常可以達到毫秒級精度。

## 後台執行

```bash
sleep 60 &  # 後台執行，不阻塞
```

## 秒 vs 浮點數

```bash
sleep 0.1   # 100 毫秒
sleep .1     # 同上（某些 shell）
sleep 100ms  # GNU sleep 支援
sleep 1s     # GNU sleep 支援
```

## Bash 內建 vs 外部程式

`sleep` 通常是 shell 內建（在 bash、zsh 中），但也存在 `/bin/sleep` 外部程式。

## 錯誤處理

```rust
if args.len() < 2 {
    eprintln!("usage: sleep seconds");
    std::process::exit(1);
}
```

不帶參數時報錯。

## 底層系統呼叫

Rust 的 `thread::sleep` 底層使用：
- `nanosleep()`：POSIX 睡眠
- `select()`：帶超時的 I/O（某些實現）

## 效能考量

`sleep` 在執行期間不消耗 CPU，是作業系統的免費等待時間。

## 實用範例

```bash
# 模擬「按任意鍵繼續」
echo "Press Enter to continue"
read
sleep 1

# 分步安裝
make -j4
sleep 2
make install

# 動畫延遲
for char in / - \\ |; do
    printf "\r%c" "$char"
    sleep 0.5
done
```

## 與 read 的結合

```bash
echo "Waiting 10 seconds..."
sleep 10
echo "Done!"
```

## 無限迴圈

```bash
while true; do
    # 做一些事
    sleep 1
done
```

## 相關指令

- `usleep`：微秒睡眠（已廢棄）
- `nohup`：執行不影響的程式
- `wait`：等待背景程序
- `at`：在指定時間執行