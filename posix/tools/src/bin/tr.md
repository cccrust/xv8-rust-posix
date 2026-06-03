# tr — 字元轉換或刪除

`tr`（translate）用於對 stdin 的字元進行轉換、刪除或壓縮。

## 核心概念

`tr` 是 stream editor，作用於字元級別而非行或單詞。

## 字元集展開

```rust
fn expand_set(s: &str) -> Vec<u8> {
    let mut set = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
            for c in bytes[i]..=bytes[i + 2] {
                set.push(c);
            }
            i += 3;
        } else {
            set.push(bytes[i]);
            i += 1;
        }
    }
    set
}
```

`a-z` 展開為所有從 'a' 到 'z' 的字元。

## 選項解析

```rust
let mut delete = false;       // -d：刪除
let mut squeeze = false;      // -s：壓縮重複
let mut complement = false;    // -c：取補集
```

## 刪除模式（-d）

```rust
if delete {
    let mut map = [true; 256];
    for &b in &active { map[b as usize] = false; }
    for &b in &input {
        if map[b as usize] { mid.push(b); }
    }
}
```

將 set1 中指定的字元從輸入中移除。

## 轉換模式

```rust
if !set2_bytes.is_empty() {
    let mut map = [0u8; 256];
    for i in 0..=255u8 { map[i as usize] = i; }
    for (i, &b) in active.iter().enumerate() {
        map[b as usize] = if i < set2_bytes.len() { set2_bytes[i] } else { *set2_bytes.last().unwrap_or(&b) };
    }
    for &b in &input {
        mid.push(map[b as usize]);
    }
}
```

set1 中的每個字元對應替換為 set2 中相同位置的字元。

## 補集模式（-c）

```rust
let active = if complement {
    let mut all: Vec<u8> = (0..=255).collect();
    all.retain(|b| !set1_bytes.contains(b));
    all
} else {
    set1_bytes.clone()
};
```

使用 set1 的補集（即所有不在 set1 中的字元）。

## 壓縮（-s）

```rust
if squeeze {
    let mut out = Vec::new();
    let mut prev: Option<u8> = None;
    for &b in &mid {
        let in_set = active.contains(&b);
        if in_set && prev == Some(b) { continue; }
        out.push(b);
        prev = Some(b);
    }
}
```

將連續重複的字元壓縮為一個。

## 典型用途

### 大小寫轉換
```bash
echo "Hello World" | tr 'a-z' 'A-Z'
# 輸出：HELLO WORLD

echo "HELLO" | tr 'A-Z' 'a-z'
# 輸出：hello
```

### 刪除特定字元
```bash
echo "hello123world456" | tr -d '0-9'
# 輸出：helloworld

echo "hello world" | tr -d ' '
# 輸出：helloworld
```

### 壓縮連續重複
```bash
echo "hello    world" | tr -s ' '
# 輸出：hello world

echo "aabbaa" | tr -s 'a'
# 輸出：aba
```

### 補充字元集
```bash
# 刪除所有非數字
echo "abc123def456" | tr -cd '0-9'
# 輸出：123456
```

## 字元類別

`tr` 支援一些預設的字元類別（完整版本）：
- `[:alnum:]`：所有字母和數字
- `[:alpha:]`：所有字母
- `[:digit:]`：所有數字
- `[:lower:]`：所有小寫字母
- `[:upper:]`：所有大寫字母
- `[:space:]`：所有空白字元

```bash
echo "HELLO world" | tr '[:upper:]' '[:lower:]'
# 輸出：hello world
```

## 與 sed 的比較

```bash
# tr 刪除
tr -d 'abc'

# sed 刪除
sed 's/[abc]//g'
```

`tr` 更簡潔，但 `sed` 更強大（支援正則表達式）。

## 與 dd 的比較

`dd` 也可以做字元轉換，但用途不同：
- `tr`：字元級替換
- `dd`：區塊級複製

## 常見範例

```bash
# 移除 CR（Windows 換行轉 Unix）
dos2unix file.txt 2>/dev/null || tr -d '\r' < file.txt > file_unix.txt

# 只保留字母和數字
cat /dev/urandom | tr -dc 'a-zA-Z0-9' | head -c 16

# 將 Tab 換成空格
tr '\t' ' ' < file.txt

# 壓縮所有連續空白為單個空格
cat file.txt | tr -s ' \t'
```

## 限制

- `tr` 不能直接讀取檔案，需要 stdin
- 輸出到 stdout，無法和 `tee` 一樣寫入多個檔案
- 不能處理多字元模式（如 "abc" → "xyz"）

## 底層系統呼叫

- `read(0, buf, n)`：從 stdin 讀取
- `write(1, buf, n)`：寫入 stdout

## 效能特點

`tr` 是非常高效的工具，因為：
1. 簡單的字元映射
2. 單次 pass
3. 最小記憶體使用

## 與 Perl 的比較

```bash
# tr 壓縮
echo "aabbaa" | tr -s 'a'

# Perl
echo "aabbaa" | perl -pe 's/(.)\1+/$1/g'
```

Perl 更靈活但開銷更大。

## 相關指令

- `sed`：流編輯器（更強大）
- `awk`：文字處理語言
- `dd`：資料複製和轉換