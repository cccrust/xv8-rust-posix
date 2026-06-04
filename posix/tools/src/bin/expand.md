# expand — Tab 轉換為 Space

`expand` 將文字中的 Tab 字元轉換為空格。

## 核心設計

```rust
for c in line.chars() {
    if c == '\t' {
        let spaces = tabs - (out.len() % tabs);
        out.push_str(&" ".repeat(spaces));
    } else {
        out.push(c);
    }
}
```

Tab 停在下一個 8 的倍數欄位（預設）。

## Tab 停止點

```
Tab 位置：  1----8----16---24---32
            ├────┤├────┤├────┤
欄位：      0         8        16
```

`\t` 輸出 enough spaces 到达下一个 stop。

## 欄位計算

```rust
let spaces = tabs - (out.len() % tabs);
```

- 當前位置 0：`8 - (0 % 8) = 8` spaces
- 當前位置 5：`8 - (5 % 8) = 3` spaces
- 當前位置 8：`8 - (8 % 8) = 0` → 重置為 8

## Tab 大小

```rust
let mut tabs = 8;  // -i 指定
```

可自訂 Tab 寬度：

```bash
expand -i 4 file.txt    # 4 spaces per tab
expand -i 2 file.txt    # 2 spaces per tab
```

## -t 選項

```rust
match c {
    't' => initial = true,  // 只有前導 tab
    'i' => { i += 1; if i < args.len() { tabs = args[i].parse().unwrap_or(8); } }
}
```

- `-t N`：設定 Tab 寬度為 N
- `-t`（僅在 xv8 版本）：只有前導 Tab

## 前導 Tab vs 全部

```rust
if initial {
    // Only expand leading tabs
    for c in line.chars() { ... }
} else {
    // Expand all tabs
    for c in line.chars() { ... }
}
```

## 典型用途

### 標準化文字
```bash
expand tabbed.txt > expanded.txt
```

### 準備列印
```bash
expand -t 8 source.c | lpr
```

### 檢視差異
```bash
expand file1.txt > /tmp/f1.txt
expand file2.txt > /tmp/f2.txt
diff /tmp/f1.txt /tmp/f2.txt
```

## 與 unexpand 的關係

| 工具 | 轉換 |
|------|------|
| `expand` | Tab → Space |
| `unexpand` | Space → Tab |

## Tab 的問題

- 不同環境 Tab 寬度不同
- `expand` 確保一致的呈現

```bash
# 原始
line with tab
12345678
     ├─ tab

# expand 後
line with tab
12345678
        ├─ 8 spaces
```

## 底層系統呼叫

`expand` 使用：
- `read()`：讀取輸入
- `write()`：寫入輸出

## 實用範例

```bash
# 基本用法
expand input.txt > output.txt

# 指定 Tab 寬度
expand -i 4 input.txt

# stdin
cat file.txt | expand
```

## 效能

`expand` 是流式處理，記憶體效率高。

## 與 sed 的比較

```bash
# sed 方式
sed 's/\t/        /g' file  # 固定 8 spaces

# expand
expand file
```

`expand` 計算正確的空格數，sed 只是簡單替換。

## 相關指令

- `unexpand`：Space → Tab
- `cat`：連接/顯示
- `fold`：行折叠
- `pr`：分頁格式化