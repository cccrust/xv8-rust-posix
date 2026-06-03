# cat — 串聯並顯示檔案內容

`cat`（concatenate）是 POSIX 最基本也是最常用的工具之一，用於讀取一個或多個檔案並將其內容輸出到標準輸出。

## 設計原理

`cat` 的核心職責只有一個：讀取檔案並逐字節輸出。它不做任何修改或過濾，單純作為資料流的管道。這種簡單性使其成為其他工具的基礎構建模組。

## 讀取策略

`cat` 支援三種輸入來源：

1. **普通檔案**：透過 `File::open` 開啟並讀取
2. **標準輸入**：當沒有指定檔案或指定 `-` 時，從 stdin 讀取
3. **多個檔案**：依序處理每個檔案

```rust
// 檔案讀取
let file = File::open(path)?;
let reader = io::BufReader::new(file);

// 標準輸入讀取
let stdin = io::stdin();
let reader = stdin.lock();
```

使用 `BufReader` 包裝檔案，利用緩衝區減少系統呼叫次數，提升讀取效率。

## 選項處理

```rust
let mut number = false;        // -n：所有行都加上行號
let mut number_nonblank = false; // -b：非空白行加行號
let mut squeeze = false;       // -s：連續空白行合併成一行
```

選項解析採用字元逐一匹配模式：
```rust
for c in args[i][1..].chars() {
    match c {
        'n' => number = true,
        'b' => number_nonblank = true,
        's' => squeeze = true,
        _ => { eprintln!("cat: invalid option -- '{}'", c); std::process::exit(1); }
    }
}
```

## 行號處理

`number` 模式下，每行輸出前會加上 6 位元組右對齊的行號加 tab：

```rust
if number || (number_nonblank && !is_blank) {
    write!(out, "{:>6}\t", line_no)?;
    line_no += 1;
}
```

`number_nonblank`（`-b`）則只對非空白行編號，這是 GNU cat 和 BSD cat 的常見行為。

## 空白行合併（squeeze）

`-s` 選項實現連續空白行合併：

```rust
if squeeze && is_blank {
    if prev_blank { continue; }  // 上一行也是空白，這行跳過
    prev_blank = true;
} else {
    prev_blank = false;
}
```

`prev_blank` 追蹤上一行是否為空白。只有連續空行的第一行被輸出，後續空白行都被忽略。

## 錯誤處理

`cat` 的錯誤處理策略：
- 檔案不存在：輸出錯誤訊息到 stderr，繼續處理下一個檔案
- 讀取失敗：同上，但最終 exit code 為非零
- stdin 讀取失敗：立即終止

```rust
if let Err(e) = cat_file(Path::new(fname), number, number_nonblank, squeeze) {
    eprintln!("cat: {}: {}", fname, e);
    std::process::exit(1);
}
```

## 底層系統呼叫

`cat` 依賴以下 POSIX 系統呼叫：

- `open(path, O_RDONLY)`：開啟檔案
- `read(fd, buf, n)`：讀取資料
- `close(fd)`：關閉檔案描述符

在 xv8 的 `libc-compat` 實作中，這些透過 `ecall` 指令發起。

## 與管道結合

`cat` 經常作為管道的一環：

```bash
cat file | grep pattern | sort | uniq
cat file | wc -l
cat file | head -n 10
```

這種組合展示了 Unix「小工具、大任務」的設計哲學。

## 效能考量

標準 `cat` 的瓶頸在於：
1. 頻繁的系統呼叫（即使使用 BufReader）
2. 每次 write 只寫一小段資料

在 Linux 上，更高效的做法是使用 `sendfile` 系統呼叫，直接在檔案描述符之間傳輸資料，避免使用者/核心空間的資料複製。

## 相關指令

- `tac`：反向顯示檔案（最後一行到第一行）
- `head`：顯示檔案開頭
- `tail`：顯示檔案結尾
- `more`/`less`：分頁檢視