# uniq — 報告或省略重複行

`uniq` 用於對相鄰的重複行進行處理：報告它們、計數它們、或只保留唯一的。

## 核心設計

`uniq` 的設計前提：**輸入必須先排序**。因為它只比較相鄰的行。

```rust
for line in &lines {
    if prev.as_deref() == Some(line.as_str()) {
        run_count += 1;  // 與上一行相同
        continue;
    }
    // 上一行的內容處理完了
    if let Some(p) = prev.take() {
        emit(&p, run_count, count, repeated, unique);
    }
    prev = Some(line.clone());
    run_count = 1;
}
```

## 計數機制

`run_count` 追蹤連續相同行的數量：

```rust
let mut run_count = 0usize;
for line in &lines {
    if prev.as_deref() == Some(line.as_str()) {
        run_count += 1;
        continue;
    }
    // 遇到不同的行，輸出上一個的計數
    if let Some(p) = prev.take() {
        emit(&p, run_count, ...);
    }
    prev = Some(line.clone());
    run_count = 1;
}
```

## 輸出模式

```rust
fn emit(line: &str, count: usize, show_count: bool, repeated: bool, unique: bool) {
    let is_rep = count > 1;  // 是否重複（出現多次）
    // 根據選項決定是否輸出
    if (repeated && !is_rep) || (unique && is_rep) { return; }
    if show_count {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}
```

## 選項解析

```rust
let mut count = false;     // -c：顯示每行出現次數
let mut repeated = false;   // -d：只顯示重複的行
let mut unique = false;      // -u：只顯示不重複的行
```

### -c（count）
輸出 `{count} {line}` 格式。

### -d（duplicate）
只輸出重複出現的行（即出現次數 > 1）。

### -u（unique）
只輸出出現一次的行（即出現次數 = 1）。

## 組合邏輯

```rust
if (repeated && !is_rep) || (unique && is_rep) { return; }
// 等價於：
// -d: !is_rep → 不輸出
// -u: is_rep → 不輸出
```

## 與 sort -u 的比較

```bash
sort file | uniq     # 先排序再去重
sort -u file        # sort 內建去重
```

兩者結果相同，但 `sort -u` 是 single-pass，`sort | uniq` 是 two-pass。

## 輸入來源

```rust
let input: Box<dyn BufRead> = if i < args.len() {
    Box::new(std::io::BufReader::new(std::fs::File::open(&args[i])?))
} else {
    Box::new(std::io::stdin().lock())
};
```

支援檔案或 stdin。

## 記憶體考量

`uniq` 需要將整個輸入讀入記憶體（或至少保留前一行的上下文）。對於超大檔案：
- 使用 `sort -u` 結合 `-T` 臨時檔案
- 使用外部排序

## 底層系統呼叫

`uniq` 本身只做比較和輸出，不需要特殊的系統呼叫。底層仍是標準備 I/O。

## 與其他工具的結合

```bash
# 統計每個唯一值的出現次數
sort file | uniq -c | sort -rn

# 找出不重複的行
sort file | uniq -u

# 找出重複的行
sort file | uniq -d

# 統計不重複的行數
sort file | uniq | wc -l
```

## 計數排序

```bash
sort file | uniq -c | sort -rn | head -n 10
```

這是一個常見的模式：統計並排序頻率最高的元素。

## 欄位去重

`uniq` 本身不支援按欄位去重，但可以結合 `awk`：

```bash
awk '{print $2}' file.txt | uniq  # 按第二欄去重
```

## 位元組 vs 字元

`uniq` 在嚴格模式下按位元組比較。多位元組字元（UTF-8 中文）可能被錯誤處理。

## 實用範例

```bash
# 基本去重
sort file.txt | uniq

# 去重並計數
sort file.txt | uniq -c

# 只顯示重複行
sort file.txt | uniq -d

# 只顯示不重複行
sort file.txt | uniq -u

# 忽略大小寫
sort file.txt | uniq -i
```

## 相關指令

- `sort -u`：帶去重的排序
- `wc -l`：計數行數
- `awk '!seen[$0]++'`：不使用 uniq 的去重方式