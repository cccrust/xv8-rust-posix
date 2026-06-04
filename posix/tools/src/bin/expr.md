# expr — 表達式求值器

`expr`（evaluate expression）用於對字串和整數進行表達式求值。

## 支援的運算

`expr` 支援多種運算：

### 1. 字串運算

```rust
// length：字串長度
if args[1] == "length" {
    println!("{}", args[2].len());
}

// substr：子字串
if args[1] == "substr" {
    let s = &args[2];
    let pos: usize = args[3].parse().unwrap_or(1);
    let len: usize = args[4].parse().unwrap_or(1);
    let result = expr_substr(s, pos, len);
    println!("{}", result);
}

// index：字元位置
if args[1] == "index" {
    let result = expr_index(&args[2], &args[3]);
    println!("{}", result);
}
```

### 2. 整數算術運算

```rust
// +, -：加法和減法
"+" => { left = left.wrapping_add(right); }
"-" => { left = left.wrapping_sub(right); }

// *, /, %：乘法、除法、餘數
"*" => { left = left.wrapping_mul(right); }
"/" => { if right == 0 { return 0; } left / right; }
"%" => { if right == 0 { return 0; } left % right; }
```

### 3. 比較運算

```rust
"=" | "==" => { if left == right { 1 } else { 0 } }
"!=" => { if left != right { 1 } else { 0 } }
">"  => { if left > right { 1 } else { 0 } }
">=" => { if left >= right { 1 } else { 0 } }
"<"  => { if left < right { 1 } else { 0 } }
"<=" => { if left <= right { 1 } else { 0 } }
```

### 4. 邏輯運算

```rust
"|"  => { if left == 0 { right } else { left } }
"&"  => { if left != 0 && right != 0 { right } else { 0 } }
"!"  => { if left == 0 { 1 } else { 0 } }
```

## 遞迴下降解析器

`expr` 使用遞迴下降來解析表達式：

```rust
parse_expr    // 處理 | (or)
parse_term    // 處理 + , -
parse_cmp     // 處理比較
parse_factor  // 處理 * , / , %
parse_primary // 處理數字和括號
```

優先順序（從低到高）：
1. `|` (lowest)
2. `&`
3. `=` `!=` `<` `<=` `>` `>=`
4. `+` `-`
5. `*` `/` `%` (highest)
6. 數字和括號

## 子字串運算

```rust
fn expr_substr(s: &str, pos: usize, len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if pos == 0 || pos > chars.len() {
        return String::new();
    }
    let start = pos - 1;  // expr 使用 1-based 索引
    let end = (start + len).min(chars.len());
    chars[start..end].iter().collect()
}
```

`expr` 的位置是 **1-based**（從 1 開始），不同於 Rust 的 0-based。

## INDEX 函數

```rust
fn expr_index(s: &str, chars_to_find: &str) -> usize {
    for (i, c) in s.chars().enumerate() {
        if chars_to_find.contains(c) {
            return i + 1;  // 1-based
        }
    }
    0  // 找不到
}
```

返回第一個匹配字元的位置（1-based），找不到返回 0。

## 退出碼

`expr` 的退出碼有特殊含義：
- `0`：表達式求值結果為非零
- `1`：表達式求值結果為零
- `2`：語法錯誤

```rust
println!("{}", result);
if result == 0 {
    std::process::exit(1);  // 結果為 0 返回 1
}
```

## 典型用途

### 算術計算
```bash
expr 10 + 20    # 輸出 30
expr 10 \* 3     # 輸出 30（需要轉義 *）
expr 30 / 4      # 輸出 7（整數除法）
```

### 字串操作
```bash
expr length "hello"        # 輸出 5
expr substr "hello" 2 3      # 輸出 ell
expr index "hello" l        # 輸出 3
```

### 條件表達
```bash
# Shell 中的條件
if expr "$a" = "$b" > /dev/null; then
    echo "equal"
fi
```

### 變數遞增
```bash
i=1
i=$(expr $i + 1)  # i = 2
```

## 與其他工具的比較

| 工具 | 用途 |
|------|------|
| `expr` | 整數算術、字串操作 |
| `bc` | 浮點數算術 |
| `awk` | 複雜計算和文字處理 |
| `$(( ))` | Bash 內建算術（更快） |

## `$(( ))` 與 expr

```bash
# expr
i=$(expr $i + 1)

# Bash 內建（推薦）
i=$((i + 1))
```

兩者功能相似，但 `$(( ))` 是 shell 內建，不需要 fork 新程序。

## 安全性

使用 `expr` 時注意：
- 乘法需要轉義 `\*`
- 結果為 0 時返回退出碼 1（與 true/false 相反）

## 底層系統呼叫

`expr` 本身不直接使用系統呼叫，只是使用者空間的計算。

## 實用範例

```bash
# 計算長度
len=$(expr length "$string")

# 字串截取
name=$(expr substr "$path" 1 3)

# 數值運算
result=$(expr 10 + 5)

# 條件（配合 && 或 ||）
expr 5 ">" 3 && echo "greater"
```

## POSIX 規範

`expr` 是 POSIX 標準的一部分，定義了：
- 整數運算
- 字串函數
- 布林運算

## 相關指令

- `bc`：任意精度計算機
- `dc`：逆波蘭表示計算機
- `$(( ))`：Shell 內建算術