# cut — 擷取檔案中的欄位或位元組

`cut` 用於從每行中取出特定部分：指定的位置範圍、欄位或位元組。

## 運作模式

`cut` 有三種主要模式：
1. **欄位模式（-f）**：根據分隔符號提取欄位
2. **字元模式（-c）**：根據字元位置提取
3. **位元組模式（-b）**：根據位元組位置提取

## 欄位解析

```rust
let parts: Vec<&str> = if delim == 0 {
    line.split_whitespace().collect()
} else {
    line.split(|c| c == delim as char).collect()
};
let out: Vec<&str> = fields.iter().filter_map(|&f| {
    if f > 0 && f <= parts.len() { Some(parts[f - 1]) } else { None }
}).collect();
```

欄位從 1 開始編號（而不是 0）。

## 欄位範圍解析

```rust
fn parse_range(s: &str) -> Vec<usize> {
    let mut ranges = Vec::new();
    for part in s.split(',') {
        if let Some((start, end)) = part.split_once('-') {
            let lo: usize = start.parse().unwrap_or(1);
            let hi: usize = if end.is_empty() { usize::MAX } else { end.parse().unwrap_or(lo) };
            for v in lo..=hi.min(lo + 1000) {
                ranges.push(v);
            }
        }
    }
    ranges
}
```

支援格式：
- `N`：第 N 個欄位
- `N-M`：第 N 到 M 個欄位
- `N-`：第 N 個到最後
- `-M`：開頭到第 M 個

## 分隔符號（-d）

```rust
let mut delim: u8 = b'\t';  // 預設是 Tab
'd' => {
    if j + 1 < arg.len() {
        delim = chars_vec[j + 1] as u8;
    } else {
        i += 1;
        if i < args.len() { delim = args[i].as_bytes()[0]; }
    }
}
```

`cut -d` 指定自訂分隔符號（預設為 Tab）。

## 字元 vs 位元組

```rust
} else if !chars.is_empty() || !bytes.is_empty() {
    let indices = if !chars.is_empty() { &chars } else { &bytes };
    let cs: Vec<char> = line.chars().collect();
    let out: String = indices.iter().filter_map(|&f| {
        if f > 0 && f <= cs.len() { Some(cs[f - 1]) } else { None }
    }).collect();
}
```

- `cut -c`：按 Unicode 字元計數（多位元組字元如中文算一個）
- `cut -b`：按位元組計數（中文每字 3 位元組）

## 與 awk 的比較

```bash
# cut 提取欄位
cut -d: -f1 /etc/passwd

# awk 提取欄位
awk -F: '{print $1}' /etc/passwd
```

`cut` 語法更簡潔，但 `awk` 更強大。

## 典型用途

### 提取欄位
```bash
# 提取用戶名
cut -d: -f1 /etc/passwd

# 提取第一和第三個欄位
cut -d' ' -f1,3 file.txt
```

### 提取字元位置
```bash
# 提取第 1-10 個字元
cut -c1-10 file.txt

# 提取第 1-5 和 10-15 個字元
cut -c1-5,10-15 file.txt
```

### 配合其他工具
```bash
# 從 ps 輸出提取 PID
ps aux | tail -n +2 | cut -d' ' -f1

# 從 ls -l 提取檔案名
ls -l | tail -n +2 | cut -d' ' -f8-
```

## 多欄位範圍

```bash
# 第 2 到第 5 欄
cut -f2-5 file.txt

# 前 5 欄
cut -f-5 file.txt

# 從第 3 欄到最後
cut -f3- file.txt
```

## 輸出分隔符號

使用 `--output-delimiter`（完整版）可以指定輸出分隔符號。

## 抑制沒有分隔符號的行

完整版 `cut` 有 `-s`（--only-delimited）選項，不輸出不包含分隔符號的行。

## UTF-8 處理

```rust
let cs: Vec<char> = line.chars().collect();  // 按字元
// vs
let bs: Vec<u8> = line.as_bytes().to_vec();  // 按位元組
```

注意差異：
- `"中"`：1 個字元，3 個位元組
- `cut -c1`：取到 1 個字元（中）
- `cut -b1`：取到 1 個位元組（'\xe'，不完整）

## 底層系統呼叫

- `read(0, buf, n)`：讀取 stdin
- `write(1, buf, n)`：寫入 stdout

## 效能考量

`cut` 是 O(n) 的，只讀取和輸出必要部分。

## 與 paste 的比較

- `cut`：從行中提取欄位
- `paste`：將多個檔案的行合併

## 常見範例

```bash
# 提取 URL 的域名
echo "http://example.com/path" | cut -d'/' -f3

# 從時間戳提取日期
date | cut -d' ' -f2-4

# 提取固定寬度欄位
cut -c1-10,20-30 file.txt
```

## 限制

- `cut` 不能處理變長度的二進制資料
- 欄位模式要求一致的分隔符號

## 相關指令

- `awk`：更強大的欄位處理
- `sed`：行編輯器
- `paste`：合併檔案