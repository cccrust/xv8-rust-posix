# fmt — 文字格式化

`fmt` 格式化段落使每行接近指定寬度。

## 核心設計

```rust
let mut paragraph = String::new();
for line in reader.lines() {
    let line = line.unwrap_or_default();
    if line.trim().is_empty() {
        // 段落分隔
        if !paragraph.is_empty() {
            print_paragraph(&paragraph, width);
            paragraph.clear();
        }
        println!();
    } else {
        if !paragraph.is_empty() { paragraph.push(' '); }
        paragraph.push_str(line.trim());
    }
}
```

將相鄰行合併為段落。

## 段落重排

```rust
fn print_paragraph(text: &str, width: usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut line = String::new();
    for word in words {
        if line.len() + word.len() + 1 > width && !line.is_empty() {
            println!("{}", line);
            line.clear();
        }
        if !line.is_empty() { line.push(' '); }
        line.push_str(word);
    }
}
```

貪心演算法：盡量填滿每行。

## 寬度預設

```rust
let mut width: usize = 75;  // 預設 75
```

## 典型用途

### 格式化文字
```bash
fmt paragraph.txt
```

### 統一 Markdown
```bash
fmt -w 80 README.md
```

### 清理腳本輸出
```bash
cat misformatted.txt | fmt
```

## 與 fold 的比較

| 工具 | 行為 |
|------|------|
| `fmt` | 重排段落，保持單字 |
| `fold` | 簡單截斷，可能切單字 |

## 段落偵測

```bash
# 雙換行 = 段落邊界
paragraph one
with multiple lines

another paragraph
```

## -w 選項

```bash
fmt -w 60 text.txt  # 每行最多 60 字元
fmt -w 100 text.txt # 每行最多 100 字元
```

## 邊界情況

###  長單字
```
Supercalifragilisticexpialidocious
```
超過寬度時，強制輸出（貪心演算法的限制）。

### 空格處理
```rust
paragraph.push_str(line.trim());  // 去除首尾空格
```

多餘空格被正規化為單一空格。

## 底層系統呼叫

`fmt` 使用：
- `read()`：讀取輸入
- `write()`：寫入輸出

## 實用範例

```bash
# 基本格式化
fmt file.txt

# 指定寬度
fmt -w 72 file.txt

# 配合其他工具
cat mess.txt | fmt -w 80
```

## 預設寬度差異

| 工具 | 預設寬度 |
|------|----------|
| `fmt` | 75 |
| `fold` | 80 |

## 與 par 的比較

`fmt` 是簡單的格式化工具。完整段落格式化（如 `par`、`reformat`）更複雜。

## 相關指令

- `fold`：簡單截斷
- `pr`：分頁格式化
- `cat`：連接/顯示