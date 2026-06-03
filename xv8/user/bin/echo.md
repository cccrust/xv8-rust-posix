# echo — 輸出文字

echo 將參數輸出到標準輸出，是最簡單的命令之一。

## 使用方式

```bash
echo [string...]
```

## 實作

```rust
fn main(args: Args) {
    for (i, arg) in args.args_as_str().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
}
```

## 行為

- 每個參數之間用空格分隔
- 最後自動輸出換行符
- 展開環境變數（由 shell 處理）

## 範例

```bash
echo hello world          # hello world
echo "hello world"        # hello world (引號由 shell 處理)
echo $HOME                # 輸出環境變數（由 shell 展開）
```

## 與 shell 的整合

echo 通常作為 shell 內建命令實現，但在 xv8 中是獨立的外部程式。

## POSIX 相容性

- 不支援 `-n`（不換行）
- 不支援 `-e`（解釋跳脽序列）

## 相關主題

- [[sh]]：命令列解釋器