# tail — 顯示檔案結尾

`tail` 用於顯示檔案的結尾部分，預設顯示最後 10 行。

## 兩種視角

`tail` 可以從兩個方向查看：
1. **從結尾往回**：顯示最後 N 行/位元組
2. **從開頭往後**：從第 N 行/位元組開始到結尾

```rust
// 從結尾顯示
fn tail_file_from_end(path: &Path, nlines: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut all_lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        all_lines.push(line?);
    }
    let start = if all_lines.len() > nlines { all_lines.len() - nlines } else { 0 };
    for line in &all_lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

// 從開頭顯示到第 N 行
fn tail_file_from_start(path: &Path, start_line: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    for (i, line) in reader.lines().enumerate() {
        if i >= start_line {
            println!("{}", line?);
        }
    }
    Ok(())
}
```

## 行偏移語法

```rust
fn parse_count(s: &str, default: usize) -> (usize, bool) {
    if let Some(rest) = s.strip_prefix('+') {
        (rest.parse().unwrap_or(default), true)  // +N 表示從第 N 行開始
    } else {
        (s.parse().unwrap_or(default), false)   // -N 表示最後 N 行
    }
}
```

- `tail -n 20 file`：顯示最後 20 行
- `tail -n +21 file`：從第 21 行開始顯示到結尾

## 位元組模式

```rust
fn tail_bytes_from_end(path: &Path, nbytes: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let len = file.seek(SeekFrom::End(0))?;
    let start = if (len as usize) > nbytes { len as usize - nbytes } else { 0 };
    let n = (len as usize - start).min(nbytes);
    let mut buf = vec![0u8; n];
    file.seek(SeekFrom::Start(start as u64))?;
    file.read_exact(&mut buf)?;
    io::stdout().write_all(&buf)?;
    Ok(())
}
```

`seek(SeekFrom::End(0))` 移動到檔案結尾，可以計算檔案大小。

## 從結尾讀取的優化

完整的 `tail -f`（follow）需要即時監控檔案增長，這在 xv8 中尚未實現。

## stdin 處理

```rust
let stdin = io::stdin();
let mut all_lines: Vec<String> = Vec::new();
for line in stdin.lock().lines() {
    all_lines.push(line.unwrap_or_default());
}
```

注意：stdin 無法 `seek`，所以需要先讀取所有行到記憶體。

## 多檔案處理

```rust
for (idx, fname) in files.iter().enumerate() {
    if files.len() > 1 {
        println!("{}==> {} <==", if idx > 0 { "\n" } else { "" }, fname);
    }
    // 根據不同模式呼叫對應函式
}
```

## 選項處理

```rust
let mut lines: usize = 10;
let mut bytes: Option<(usize, bool)> = None;
let mut from_start = false;
```

- `-n N`：顯示 N 行
- `-c N`：顯示 N 位元組
- `+N`：從第 N 行/位元組開始

## 與 head 的對稱性

`head` 和 `tail` 是互補的：
- `head`：從開頭
- `tail`：從結尾

```bash
# 看第 1-10 行
head -n 10 file.txt

# 看第 11-20 行
tail -n +11 file.txt | head -n 10
```

## 實用範例

```bash
# 顯示最後 20 行日誌
tail -n 20 /var/log/syslog

# 從第 100 行開始顯示到結尾
tail -n +100 largefile.txt

# 顯示最後 50 位元組
tail -c 50 binary.dat

# 即時監控日誌（GNU tail）
tail -f /var/log/nginx/access.log
```

## 與其他工具的組合

```bash
# 看倒數第 2 行
tail -n 2 file.txt | head -n 1

# 排除最後 1 行
head -n -1 file.txt

# 顯示最後 5 行並持續監控
tail -n 5 -f app.log
```

## 底層系統呼叫

- `open`：開啟檔案
- `seek`：移動檔案指標
- `read`：讀取資料
- `close`：關閉檔案

## 效能考量

1. **從結尾讀取**：需要 `seek`，但仍是 O(n) 的
2. **大檔案**：`tail -n 5` 仍需要讀取整個檔案
3. **優化版本**：`tail` 會從結尾開始反覆讀取 blocks

## GNU tail 的功能

- `-f`：follow 模式，持續輸出新增內容
- `-s N`：配合 -f，每 N 秒檢查一次
- `--retry`：持續嘗試開啟檔案（適用於輪轉日誌）

## 常見用法

```bash
# 動態查看日誌
tail -f /var/log/auth.log

# 監控錯誤
tail -f /var/log/nginx/error.log | grep ERROR

# 持續監控並高亮特定內容
tail -f logfile | grep --line-buffered PATTERN
```

## 相關指令

- `head`：顯示開頭
- `tac`：反向顯示檔案
- `less`：互動式檢視（`F` 命令相當於 tail -f）