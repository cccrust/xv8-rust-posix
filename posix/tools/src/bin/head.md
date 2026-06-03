# head — 顯示檔案開頭

`head` 用於顯示檔案的開頭部分，預設顯示前 10 行。

## 核心設計

`head` 的實作有兩種模式：
1. **行模式**：顯示前 N 行
2. **位元組模式**：顯示前 N 個位元組

```rust
fn head_file(path: &Path, lines: usize, bytes: Option<usize>) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    if let Some(nbytes) = bytes {
        // 位元組模式
        let mut buf = vec![0u8; nbytes];
        let mut handle = reader.into_inner();
        let n = std::io::Read::read(&mut handle, &mut buf)?;
        io::stdout().write_all(&buf[..n])?;
    } else {
        // 行模式
        for (i, line) in reader.lines().enumerate() {
            if i >= lines { break; }
            println!("{}", line?);
        }
    }
}
```

## 行計數

行模式下，到達指定行數後立即停止讀取：

```rust
for (i, line) in reader.lines().enumerate() {
    if i >= lines { break; }
    println!("{}", line?);
}
```

這比讀取整個檔案再輸出前 N 行更有效率。

## 位元組模式

```rust
let mut buf = vec![0u8; nbytes];
let mut handle = reader.into_inner();
let n = std::io::Read::read(&mut handle, &mut buf)?;
io::stdout().write_all(&buf[..n])?;
```

注意：`BufReader::into_inner()` 取回底層 `File`，以便進行原始位元組讀取。

## 選項解析

```rust
let mut lines: usize = 10;      // -n：行數
let mut bytes: Option<usize> = None;  // -c：位元組數
```

## 多檔案處理

```rust
for (idx, fname) in files.iter().enumerate() {
    if files.len() > 1 {
        println!("{}==> {} <==", if idx > 0 { "\n" } else { "" }, fname);
    }
    head_file(path, lines, bytes)?;
}
```

當有多個檔案時，在輸出每個檔案前加上 header：

```
==> file1.txt <==
行1
行2

==> file2.txt <==
行1
行2
```

## stdin 處理

```rust
if files.is_empty() {
    let reader = io::stdin();
    for (i, line) in reader.lock().lines().enumerate() {
        if i >= lines { break; }
        println!("{}", line.unwrap_or_default());
    }
}
```

`-` 也可以明確表示 stdin：
```bash
cat file.txt | head -n 5
head -n 5 - < file.txt
```

## 選項 `-q` 和 `-v`

- `-q`（quiet/silent）：不輸出檔案 header
- `-v`（verbose）：總是輸出 header

xv8 的實現目前忽略這些選項，但標準行為是：
```bash
head -v file.txt  # 總是輸出 ==> file.txt <==
head -q file.txt  # 不輸出 header
```

## 與 tail 的比較

`tail` 顯示檔案結尾，與 `head` 互補：
- `head`：從開頭
- `tail`：從結尾

## 實用範例

```bash
# 顯示前 20 行
head -n 20 file.txt

# 顯示前 100 位元組
head -c 100 file.txt

# 查看檔案開頭（不用 wc 就知道行數）
head -n 1 file.txt  # 只看第一行

# 多個檔案
head -n 5 file1.txt file2.txt
```

## 與其他工具的組合

```bash
# 只看檔案的第 1-5 行（結合 tail）
head -n 5 file.txt | tail -n 5

# 顯示第 1 行
head -n 1 file.txt

# 從大型日誌的開頭看起
head -n 100 /var/log/syslog
```

## 底層系統呼叫

- `open(path, O_RDONLY)`：開啟檔案
- `read(fd, buf, n)`：讀取資料
- `close(fd)`：關閉檔案描述符

## 效能特點

`head` 的優勢在於：
1. **提前終止**：找到足夠行數後立即停止
2. **最小記憶體**：不需要讀取整個檔案
3. **流式處理**：可以處理無窮流

## 與 `sed` 的比較

```bash
head -n 5 file.txt    # 簡潔
sed -n '1,5p' file.txt # 更靈活但複雜
```

## 標準輸入的管道

```bash
# 查看命令輸出
ls -la | head -n 10

# 查看過程輸出
make | head -n 20
```

## 相關指令

- `tail`：顯示結尾
- `sed`：流編輯器（可選取特定行）
- `awk`：更強大的文字處理