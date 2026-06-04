# paste — 合併行

`paste` 將多個檔案的行並排合併。

## 核心設計

```rust
while !done {
    let mut parts = Vec::new();
    for (fi, lines) in all_lines.iter().enumerate() {
        if iterators[fi] < lines.len() {
            parts.push(lines[iterators[fi]].clone());
        }
    }
    if !parts.is_empty() {
        println!("{}", parts.join(&delim.to_string()));
    }
    // 遞進每個檔案的迭代器
}
```

並行讀取多個檔案的同一行。

## 平行模式

```bash
paste file1.txt file2.txt file3.txt
```

輸出：
```
line1_f1	line1_f2	line1_f3
line2_f1	line2_f2	line2_f3
```

## -d 分隔符

```rust
let mut delim = '\t';
if args[i] == "-d" && i + 1 < args.len() {
    delim = args[i + 1].chars().next().unwrap_or('\t');
}
```

```bash
paste -d',' file1.txt file2.txt
# 輸出：
# line1_f1,line1_f2
# line2_f1,line2_f2
```

## -s 序列模式

```rust
if serial {
    // file1 的所有行 → file2 的所有行 → ...
}
```

```bash
paste -s file.txt
# 輸出：
# line1	line2	line3	...
```

一次處理一個檔案的所有行。

## 分隔符字元

```bash
paste -d': ' file1 file2
# 輸出：
# line1: line2

paste -d'\n' file1 file2  # 換行分隔
```

多個分隔符可用陣列，但此實現只支援第一個字元。

## 典型用途

### 建立表格
```bash
paste names.txt values.txt
```

### CSV 格式
```bash
paste -d',' name.csv age.csv
```

### 水平合併
```bash
paste file1 file2 file3
```

## 與其他工具的比較

| 工具 | 方向 | 用途 |
|------|------|------|
| `paste` | 水平 | 合併多檔案的行 |
| `cat` | 垂直 | 連接多檔案 |
| `join` | 水平 | 基於欄位連接 |

## 檔案長度不同

```bash
# file1: 3 行, file2: 2 行
paste file1.txt file2.txt
# 輸出：
# line1_f1	line1_f2
# line2_f1	line2_f2
# line3_f1
```

較短的檔案用完後，該欄位留空。

## stdin 處理

```rust
let files: Vec<&str> = if i < args.len() {
    args[i..].iter().map(|s| s.as_str()).collect()
} else {
    vec!["-"]  // 預設 stdin
};
```

無參數時從 stdin 讀取。

## 底層系統呼叫

`paste` 使用：
- `read()`：讀取多個檔案
- `write()`：寫入輸出

## 實用範例

```bash
# 基本合併
paste file1.txt file2.txt

# 自訂分隔符
paste -d': ' file1 file2

# 序列模式
paste -s file.txt

# 建立列表
paste -d'|' -s items.txt
```

## 與 cut 的關係

`paste` 和 `cut` 是互補的：
- `cut -c1-10 file` → 提取欄位
- `paste -d' ' col1.txt col2.txt` → 合併欄位

## 相關指令

- `cut`：提取欄位
- `cat`：連接檔案
- `join`：欄位連接