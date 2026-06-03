# poweroff — 關閉系統

poweroff 立即關閉 xv8 系統。

## 使用方式

```bash
poweroff [exit_code]
```

## 實作

```rust
fn main(args: Args) {
    let code = args
        .get_str(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    poweroff(code);
}
```

## 行為

- 呼叫 `poweroff` 系統呼叫
- 系統進入無限迴圈
- QEMU 會隨即終止

## 與 init 的整合

poweroff 通常由 init 或 shell 在收到特定訊號時呼叫。

## 退出碼

| 退出碼 | 說明 |
|--------|------|
| 0 | 正常關閉 |
| 其他 | 錯誤關閉 |

## 相關主題

- [[init]]：第一個使用者程序