# sleep — 程式暫停

sleep 讓程式暫停指定的 tick 數。

## 使用方式

```bash
sleep ticks
```

## 實作

```rust
fn main(args: Args) {
    if args.len() != 2 {
        exit_with_msg("usage: sleep ticks");
    }

    let ticks = args.args_as_str().next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| exit_with_msg("sleep: invalid ticks"));

    if let Err(e) = sleep(ticks) {
        eprintln!("sleep: {}", e);
        exit(1);
    }
}
```

## 行為

- 參數是以 tick 為單位的整數
- 每個 tick 約 10ms（在 QEMU 中）
- 程式在睡眠期間不消耗 CPU

## 與 uptime 的關係

```bash
$ uptime          # 顯示系統運行時間（以 tick 為單位）
12345
$ sleep 100       # 暫停 100 ticks (約 1 秒)
$ uptime          # 增加約 100
12445
```

## 使用範例

```bash
sleep 10           # 暫停 10 ticks (100ms)
sleep 100          # 暫停 100 ticks (1s)
sleep 1000         # 暫停 1000 ticks (10s)
```

## 與 shell 的整合

sleep 是 shell 內建命令之一，用於：
- 延遲執行
- 速率限制
- 定時輪詢

## 與 POSIX 的差異

- 參數是 tick 而非秒
- 不支援 `sleep 1s`、`sleep 1m` 等單位

## 相關主題

- [[uptime]]：系統運行時間
- [[demo]]：使用 sleep 展示調度