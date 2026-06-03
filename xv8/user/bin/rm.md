# rm — 刪除檔案或目錄連結

rm 刪除檔案或目錄的連結（unlink）。

## 使用方式

```bash
rm file...
```

## 實作

```rust
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg("usage: rm files...");
    }

    for name in args.args_as_str() {
        if let Err(e) = unlink(name) {
            eprintln!("rm: {} ({})", e, name);
            break;
        }
    }
}
```

## 行為

- 刪除檔案的目錄項
- 如果是檔案的最後一個連結，釋放磁碟空間
- 目錄必須為空才能刪除（使用 rmdir）

## unlink vs rmdir

| 系統呼叫 | 說明 |
|----------|------|
| `unlink` | 刪除目錄項，適用於檔案 |
| `rmdir` | 刪除空目錄 |

## 錯誤處理

- 檔案不存在：`NoEntry`
- 目錄非空：`NotEmpty`
- 權限不足：`NotPermitted`

## 範例

```bash
rm file.txt
rm oldfile1.txt oldfile2.txt
```

## 安全性

- rm 不會刪除目錄（需使用 rmdir）
- 沒有 `-r` 選項遞迴刪除

## 相關主題

- [[mkdir]]：建立目錄
- [[ln]]：建立連結