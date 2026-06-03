# uptime — 顯示系統運行時間

uptime 顯示系統自開機以來的 tick 數。

## 使用方式

```bash
uptime
```

## 實作

```rust
fn main(_args: Args) {
    println!("{}", uptime());
}
```

## 輸出

```
$ uptime
12345
```

## Tick 與時間的關係

在 xv8/QEMU 中：
- 每個 tick = 10 毫秒
- `uptime = 12345` 表示系統已運行約 123.45 秒

```
時間（秒）= uptime * 10ms
```

## 使用場景

- 效能測量
- 程式計時
- 系統監控

## 與睡眠的關係

sleep 接受的參數就是 tick 數：

```bash
sleep 100   # 睡眠 100 ticks = 1 秒
```

## 相關主題

- [[sleep]]：程式睡眠
- [[demo]]：使用 uptime 測量效能