# sort — 排序文字行

`sort` 用於將文本行的順序重新排列，是文字處理中不可或缺的工具。

## 排序原理

`sort` 的核心是 Rust 的 `sort()` 方法：

```rust
lines.sort();
```

對於數值排序：
```rust
lines.sort_by(|a, b| {
    let an: f64 = a.trim().parse().unwrap_or(0.0);
    let bn: f64 = b.trim().parse().unwrap_or(0.0);
    an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
});
```

## 排序類型

### 字典排序（預設）
```rust
lines.sort();  // ASCII/字典順序
```

### 數值排序
```rust
lines.sort_by(|a, b| {
    let an: f64 = a.trim().parse().unwrap_or(0.0);
    let bn: f64 = b.trim().parse().unwrap_or(0.0);
    an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
});
```

注意這裡使用了 `parse::<f64>`，所以能處理浮點數。

## 選項處理

```rust
let mut reverse = false;   // -r：反向排序
let mut numeric = false;    // -n：數值排序
let mut unique = false;     // -u：去重
```

## 反向排序

```rust
if reverse {
    lines.reverse();
}
```

## 去重（-u 選項）

```rust
let mut prev: Option<String> = None;
for line in &lines {
    if unique {
        if prev.as_deref() == Some(line.as_str()) { continue; }
        prev = Some(line.clone());
    }
    println!("{}", line);
}
```

`-u` 等於先排序再去除相鄰的重複行。

## 讀取來源

```rust
if files.is_empty() {
    // 從 stdin 讀取
    for line in io::stdin().lock().lines() {
        lines.push(line.unwrap_or_default());
    }
} else {
    // 從檔案讀取
    for fname in files {
        let content = std::fs::read_to_string(&fname)?;
        for line in content.lines() {
            lines.push(line.to_string());
        }
    }
}
```

## 記憶體考量

`sort` 將所有行載入記憶體。對於大檔案：
- 使用 `-T` 指定臨時目錄
- 使用 `-S` 限制記憶體使用
- GNU sort 支援外部排序

## 欄位排序

完整版 `sort` 支援 `-k` 指定排序鍵：

```bash
sort -k2,2 -k3,3n data.txt  # 先按第2欄排序，再按第3欄數值排序
```

xv8 的 `sort` 目前不支援多鍵排序。

## 穩定排序

Rust 的 `sort` 是 stable 的，即相等元素保持原本順序。這對於多鍵排序很重要。

## 底層排序演算法

Rust 使用了 pattern-defeating quick sort（pdqsort），結合了：
- Quick sort 的效能
- Merge sort 的穩定性
- 避免 worst-case

## 與 uniq 的組合

```bash
sort file | uniq    # 去重（等於 sort -u）
sort file | uniq -c # 去重並計數
```

## 典型用途

```bash
# 基本排序
sort names.txt

# 數值排序
sort -n scores.txt

# 反向排序
sort -r leaderboard.txt

# 去重
sort -u duplicates.txt
```

## 多檔案處理

```bash
sort file1.txt file2.txt > combined.txt
```

所有檔案的內容合併後排序。

## 文化差異

不同 locale 影響排序順序：
- `C` locale：ASCII 順序
- `en_US.UTF-8`：大小寫不敏感
- 中文：根據 locale 可能按筆畫或拼音

## 相關指令

- `uniq`：去除相鄰重複行
- `tsort`：拓撲排序
- `shuf`：隨機排序