# fold — 行折叠

`fold` 將長行折疊為較短的行，預設每行 80 字元。

## 核心設計

```rust
while pos < bytes.len() {
    let end = (pos + width).min(bytes.len());
    // Try to break at a space if possible
    let break_at = if end < bytes.len() {
        bytes[pos..end].iter().rposition(|&b| b == b' ')
            .map(|r| pos + r + 1)
            .unwrap_or(end)
    } else {
        end
    };
    println!("{}", &line[pos..break_at]);
    pos = break_at;
}
```

嘗試在空格處斷行，避免中斷單字。

## 中斷點選擇

```rust
bytes[pos..end].iter().rposition(|&b| b == b' ')
```

從結尾向前找第一個空格，優先單字邊界斷行。

## 寬度設定

```rust
let mut width: usize = 80;  // -w 指定
```

可自訂行寬：
```bash
fold -w 60 file.txt   # 60 字元
fold -w 132 file.txt  # 132 字元
```

## 典型用途

### 預覽文字
```bash
cat longfile.txt | fold
```

### 準備列印
```bash
fold -w 80 document.txt | lpr
```

### 固定寬度顯示
```bash
fold -s -w 70  # 保持單字完整
```

## -s 選項

`fold` 實現使用 `rposition` 向後找空格，實際就是 `-s`（在空格斷行）的行為。

## 與其他工具的比較

| 工具 | 用途 |
|------|------|
| `fold` | 簡單行折叠 |
| `fmt` | 段落格式化（保持單字）|
| `pr` | 分頁 + 行號 |

## 邊界情況

### 無空格長行
```bash
echo "aaaaa..." | fold -w 10
```
會在確切寬度處中斷。

### 空白行
```bash
echo -e "line1\n\nline2" | fold -w 20
```
空白行原樣輸出。

## 底層系統呼叫

`fold` 使用：
- `read()`：讀取輸入
- `write()`：寫入輸出

## 實用範例

```bash
# 基本用法
fold file.txt

# 指定寬度
fold -w 60 file.txt

# 保持單字完整（找空格）
fold -s -w 60 file.txt

# stdin
cat file.txt | fold -w 70
```

## 位元組 vs 字元

```rust
let bytes = line.as_bytes();
```

`fold` 按位元組處理，可能在中文字元中間斷行（全寬字元需 2+ 位元組）。

## 與-pr 的比較

- `fold`：只是斷行
- `pr`：新增分頁、行號、標頭

## 相關指令

- `fmt`：段落格式化
- `cut`：提取欄位
- `pr`：分頁格式化