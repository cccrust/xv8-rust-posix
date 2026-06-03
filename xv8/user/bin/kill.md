# kill — 傳送信號給程序

kill 傳送信號（預設 SIGTERM）給指定程序。

## 使用方式

```bash
kill pid...
```

## 實作

```rust
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg("usage: kill pid...");
    }

    for pid in args.args_as_str() {
        let pid = pid.parse::<usize>().unwrap_or_else(|_| {
            exit_with_msg("kill: invalid pid");
        });
        if kill(pid).is_err() {
            eprintln!("kill: failed to kill {}", pid);
        }
    }
}
```

## 預設行為

- 傳送 SIGTERM (15) 給目標程序
- 程序可以捕捉或忽略 SIGTERM
- SIGKILL (9) 無法被捕捉或忽略

## 常用訊號

| 訊號 | 值 | 預設動作 | 說明 |
|------|-----|----------|------|
| SIGTERM | 15 | 終止 | 優雅終止請求 |
| SIGKILL | 9 | 終止 | 強制終止 |
| SIGINT | 2 | 終止 | 中斷 (Ctrl-C) |
| SIGSTOP | 19 | 暫停 | 暫停程序 |
| SIGCONT | 18 | 繼續 | 繼續暫停的程序 |

## 錯誤處理

- 程序不存在：`NoProcess`
- 權限不足：`NotPermitted`

## 範例

```bash
kill 1234           # 終止程序 1234
kill -9 1234        # 強制終止
kill -STOP 1234     # 暫停程序
kill -CONT 1234     # 繼續執行
```

## 與 shell 的整合

shell 的 `kill` 是內建命令，可以識別工作控制 ID（如 `%1`）。

## 相關主題

- [[signal]]：訊號機制