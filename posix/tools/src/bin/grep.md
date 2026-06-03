# grep — 文字搜尋工具

`grep`（Global Regular Expression Print）是 Unix 系統中最强大的文字搜尋工具之一，用於在檔案或標準輸入中搜尋符合模式的行。

## 功能概述

`grep` 的核心任務：
1. 讀取一行文字
2. 檢查該行是否符合指定的模式
3. 如果符合，輸出該行

## 模式匹配

xv8 的 `grep` 實現使用了簡單的子字串匹配：

```rust
fn matches(line: &str, pattern: &str, opts: &GrepOpts) -> bool {
    let matched = if opts.ignore_case {
        line.to_lowercase().contains(&pattern.to_lowercase())
    } else {
        line.contains(pattern)
    };
    matched != opts.invert
}
```

這不同於傳統 `grep` 的正則表達式，但對於簡單的關鍵字搜尋已足夠。

## 選項處理

```rust
struct GrepOpts {
    ignore_case: bool,       // -i：忽略大小寫
    invert: bool,            // -v：反轉匹配（輸出不相符的行）
    count: bool,             // -c：只輸出匹配的數量
    line_number: bool,       // -n：顯示行號
    files_with_matches: bool, // -l：只輸出包含匹配的文件名
}
```

## 讀取模式

`grep` 可以從多個來源讀取：
- **多個檔案**：依序搜尋每個檔案
- **stdin**：當沒有指定檔案或使用 `-` 時
- **管道**：接收前一個命令的輸出

```rust
for (i, line) in io::stdin().lock().lines().enumerate() {
    if matches(&line, pattern, &opts) {
        if opts.line_number { print!("{}:", i + 1); }
        println!("{}", line);
    }
}
```

## 行號顯示

```rust
if opts.line_number {
    print!("{}:", i + 1);  // 行號從 1 開始
}
println!("{}", line);
```

## -l 選項的短路行為

找到第一個匹配就立即返回：

```rust
if opts.files_with_matches {
    println!("{}", path.display());
    return Ok(());  // 短路返回，不繼續搜尋
}
```

這優化了效能，因為只需要確認檔案是否包含匹配。

## -c 計數模式

計數模式下只統計，不輸出匹配的行：

```rust
if opts.count {
    continue;  // 只增加計數，不輸出
}
```

最終輸出：`{count}`（數字）

## 與 egrep/fgrep 的關係

- `grep`：基本模式匹配（這裡是子字串）
- `egrep`：擴展正則表達式（Extended RE）
- `fgrep`：固定字串（Fixed string，不解釋正則）

在 POSIX 中，這些是 grep 的選項或軟連結。

## 底層系統呼叫

`grep` 主要依賴：
- `open(path, O_RDONLY)`：開啟檔案
- `read(fd, buf, n)`：讀取資料
- `close(fd)`：關閉檔案描述符

對於大檔案，更高效的實作會使用 `mmap` 來避免反覆讀取。

## 效能考量

1. **IO 優先於 CPU**：先優化讀取方式
2. **減少輸出**：`-l`、`-c` 等可以提前結束
3. **大檔案**：使用 `mmap` 或記憶體對映

## 正規表示式回顧

完整的 `grep` 支援正則表達式：
- `.`：任意單個字元
- `*`：前面元素零次或多次
- `^`：行首
- `$`：行尾
- `[...]`：字元類

```bash
grep "^root" /etc/passwd   # 以 root 開頭的行
grep "bash$" /etc/passwd   # 以 bash 結尾的行
grep "[0-9]" file          # 包含數字的行
```

## 環境變數

`GREP_OPTIONS`：歷史相容性選項（已被棄用）

## 典型用途

```bash
# 在日誌中搜尋錯誤
grep ERROR /var/log/syslog

# 找出哪些檔案包含某字串
grep -l "function" *.c

# 統計匹配行數
grep -c "error" *.log

# 反向搜尋（不包含）
grep -v "DEBUG" app.log
```

## 相關指令

- `egrep`：擴展正則
- `fgrep`：固定字串
- `ack`：專為程式設計的搜尋工具
- `ag`：更快的 Silver Searcher
- `ripgrep (rg)`：Rust 實現的高速搜尋