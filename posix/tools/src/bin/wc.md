# wc — 計算位元組、字元、行的數量

`wc`（word count）用於計算檔案中的行數、字數、位元組數和字元數。

## 計數結構

```rust
struct Counts {
    lines: usize,   // 行數
    words: usize,   // 字數
    chars: usize,   // 字元數（Unicode）
    bytes: usize,   // 位元組數
}
```

## 計數邏輯

```rust
fn count(reader: &mut impl BufRead) -> Counts {
    let mut c = Counts { lines: 0, words: 0, chars: 0, bytes: 0 };
    let mut buf = String::new();
    loop {
        buf.clear();
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                c.lines += 1;
                c.bytes += buf.len();
                c.chars += buf.chars().count();
                c.words += buf.split_whitespace().count();
            }
            Err(_) => break,
        }
    }
    c
}
```

- **行數**：計算 `read_line` 的成功次數
- **位元組數**：`String` 的長度（UTF-8 位元組數）
- **字元數**：`chars().count()`（Unicode 字元數）
- **字數**：`split_whitespace().count()`（空白分隔的單詞數）

## 位元組 vs 字元

在純 ASCII 文字中，位元組數 = 字元數。但在有多位元組字符（如 UTF-8 中文）時：
- `len()` = 3 位元組（每個中文字 3 bytes）
- `chars().count()` = 1 個字元

例如「中」字：
- `len()` = 3
- `chars().count()` = 1

## 選項處理

```rust
let mut show_lines = true;
let mut show_words = true;
let mut show_chars = false;
let mut show_bytes = true;
```

- `-l`：只顯示行數
- `-w`：只顯示字數
- `-c`：只顯示位元組數
- `-m`：只顯示字元數

注意：同時指定多個選項會替換為只顯示那些項目，而非同時顯示全部。

## 總計計算

```rust
if filenames.len() > 1 {
    print_counts(&total, "total", ...);
}
```

當有多個檔案時，會計算並輸出總計。

## stdin 處理

```rust
if opt_i == args.len() || args[opt_i] == "-" {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let c = count(&mut reader);
    print_counts(&c, "", ...);
}
```

`-` 表示從 stdin 讀取。

## 底層系統呼叫

`wc` 底層使用標準 I/O 讀取，不需要特殊的系統呼叫。但效率優化版本可能使用：
- `read(fd, buf, n)`：直接讀取位元組
- `mmap()`：記憶體對應檔案

## 效能考量

基本的 `wc` 實作是 O(n) 的，讀取整個檔案並統計。

更高效的實作使用更大的緩衝區，或使用記憶體對映避免複製。

## 輸出格式

```rust
if show_lines { print!("{:>8} ", c.lines); }
if show_words { print!("{:>8} ", c.words); }
if show_chars { print!("{:>8} ", c.chars); }
if show_bytes { print!("{:>8} ", c.bytes); }
println!("{}", filename);
```

每個數字佔 8 個字元，右對齊，後面跟一個空格。

## 與其他工具的結合

```bash
# 計算行數
ls | wc -l

# 統計某種模式的行數
grep -c "pattern" file

# 計算總行數
find . -name "*.rs" | xargs wc -l
```

## 相關指令

- `cloc`：統計程式碼行數（可區分註釋和空白）
- `sum`：計算校驗和
- `md5sum`：計算 MD5 校驗和