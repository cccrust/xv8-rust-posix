# mkdir — 建立目錄

mkdir 建立新目錄。

## 使用方式

```bash
mkdir directory...
```

## 實作

```rust
fn main(args: Args) {
    if args.len() < 2 {
        exit_with_msg("usage: mkdir directory...");
    }

    for dir in args.args_as_str() {
        if let Err(e) = mkdir(dir) {
            eprintln!("mkdir: {} ({})", e, dir);
            break;
        }
    }
}
```

## 行為

- 接受一個或多個目錄名稱
- 遇到錯誤時終止（不繼續建立後續目錄）
- 使用 `mkdir` 系統呼叫

## 錯誤處理

| 錯誤 | 說明 |
|------|------|
| NoEntry | 父目錄不存在 |
| AlreadyExists | 目錄已存在 |
| NotDirectory | 父路徑不是目錄 |

## 範例

```bash
mkdir newdir
mkdir dir1 dir2 dir3
```

## 相關主題

- [[rm]]：刪除檔案/目錄