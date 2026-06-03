# split — 分割檔案

`split` 將大檔案分割成多個較小的部分。

## 核心設計

```rust
let lines_per_file = 1000;  // 預設每檔 1000 行

let num_files = (lines.len() + lines_per_file - 1) / lines_per_file;
let digits = num_files.to_string().len().max(suffix_len);

let mut chunk = 0;
let mut out_lines = Vec::new();
for (idx, line) in lines.iter().enumerate() {
    out_lines.push(line);
    if out_lines.len() == lines_per_file || idx == lines.len() - 1 {
        let suffix = format!("{:0width$}", chunk, width = digits);
        let filename = format!("{}{}", prefix, suffix);
        // 寫入...
        chunk += 1;
    }
}
```

## 前綴和後綴

```rust
let mut prefix = "x".to_string();
let mut suffix_len = 2;
```

預設輸出：`xaa`, `xab`, `xac`, ...

## 後綴數量計算

```rust
let digits = num_files.to_string().len().max(suffix_len);
```

確保後綴長度足以容纳所有檔案編號。

## 行模式

```rust
match args[i].as_str() {
    "-l" if i + 1 < args.len() => {
        lines_per_file = s.parse().unwrap_or(1000);
    }
}
```

`-l N`：每 N 行一個檔案。

## 典型用途

### 基本分割
```bash
split bigfile.txt
# 輸出：xaa, xab, xac, ...
```

### 指定行數
```bash
split -l 500 bigfile.txt part_
# 輸出：part_aa, part_ab, ...
```

### 自訂前綴
```bash
split -d -a 4 bigfile.txt segment_
# 輸出：segment_0000, segment_0001, ...
```

## 配合連接

```bash
# 分割
split bigfile.txt

# 傳輸（多個小檔案）
cat xaa xab xac > recovered.txt
```

## 與 csplit 的比較

| 工具 | 用途 |
|------|------|
| `split` | 等大小/行數分割 |
| `csplit` | 按模式分割 |

## 安全性

`split` 只讀取和寫入，不執行其他操作。

## 底層系統呼叫

`split` 使用：
- `read()`：讀取原始檔案
- `write()`：寫入分割檔案

## 實用範例

```bash
# 分割大日誌
split -l 10000 access.log log_

# 分割二進制檔案
split -b 10M large.bin chunk_

# 數字後綴
split -d bigfile.txt part
# 輸出：part00, part01, part02, ...
```

## 恢復原檔

```bash
cat xaa xab xac > original.txt
```

注意：合併順序重要。

## 效能

`split` 需要將整個檔案讀入記憶體，對超大檔案可能需要優化。

## 選項

- `-a N`：後綴長度
- `-d`：使用數字後綴
- `-l N`：每 N 行
- `-b N`：每 N 位元組

## 相關指令

- `cat`：連接檔案
- `csplit`：按模式分割
- `paste`：並行合併