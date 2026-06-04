# cat — 串聯並輸出檔案

cat 讀取檔案並輸出到標準輸出，是 Unix 最基本也最常用的命令之一。

## 使用方式

```bash
cat [file...]
```

## 實作

```rust
fn cat(mut fd: Fd) {
    let mut buf = [0u8; 512];

    loop {
        match fd.read(&mut buf) {
            Ok(0) => break,                    // EOF
            Ok(n) => Stdout.write_all(&buf[..n]),
            Err(_) => exit_with_msg("cat: read error"),
        }
    }
}

fn main(args: Args) {
    if args.len() <= 1 {
        cat(Fd::STDIN);                       // 從 stdin 讀取
        return;
    }

    for path in args.args_as_str() {
        let Ok(fd) = open(path, OpenFlag::READ_ONLY) else {
            exit_with_msg("cat: cannot open file");
        };
        cat(fd);
        let _ = close(fd);
    }
}
```

## 行為說明

| 情況 | 行為 |
|------|------|
| 無參數 | 從 stdin 讀取 |
| 多個檔案 | 依序輸出所有檔案 |
| 檔案不存在 | 輸出錯誤訊息並退出 |

## 讀取機制

- 使用 512 位元組緩衝區
- 重複讀取直到 EOF (傳回 0)
- 讀取錯誤時終止

## 與 POSIX 的差異

- 只支援 `-` 讀取 stdin（未實現）
- 不支援 -v, -n 等選項

## 範例

```bash
cat file.txt           # 輸出檔案
cat f1 f2 f3          # 依序輸出多個檔案
cat < file.txt        # 從 stdin 讀取
```

## 相關主題

- [[ls]]：列目錄
- [[wc]]：字計數