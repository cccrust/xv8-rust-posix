# nl — 附加行號

`nl` 為檔案行附加行號。

## 核心設計

```rust
let mut lineno = start;  // 預設從 1 開始

for line in reader.lines() {
    let line = line.unwrap_or_default();
    let is_empty = line.trim().is_empty();

    if should_number {
        println!("{:>6}{}{}", lineno, sep, line);
        lineno += inc;
    } else {
        println!("{:>6}{}{}", "", sep, line);
    }
}
```

行號靠右對齊，6 位元組寬度。

## 輸出格式

```
     1	First line
     2	Second line
     3
     4	Fourth line
```

空行：
- 有行號：顯示行號
- 無行號：空白（6 個空格 + 分隔符）

## 編號選項

```rust
match c {
    'b' => number_nonempty = false,   // -b t：編號所有行
    'n' => number_all = false,         // -n ln：無行號（實際相反）
    's' => sep = "",                   // -s：行號後無分隔
    'v' => number_all = true,          // -v：重新設置
}
```

## -b 選項

```bash
nl -b t file.txt   # 編號所有行（預設）
nl -b a file.txt   # 編號所有行
nl -b n file.txt   # 不編號任何行
nl -b pREGEX file.txt  # 只編號符合正則的
```

## -n 選項

```bash
nl -n ln file.txt  # 左對齊
nl -n rn file.txt  # 右對齊（預設）
nl -n rz file.txt  # 補零
```

## -s 分隔符

```bash
nl -s ": " file.txt
# 輸出：
# 1: First line
# 2: Second line
```

## -v 起始值

```bash
nl -v 10 file.txt  # 從 10 開始
```

## -i 增量

```bash
nl -i 2 file.txt  # 每次 +2
```

## 與其他工具的比較

| 工具 | 用途 |
|------|------|
| `nl` | 附加行號 |
| `cat -n` | 顯示 + 行號 |
| `wc -l` | 計算行數 |

## 典型用途

### 程式碼行號
```bash
nl source.rs > numbered.rs
```

### 合併差異
```bash
diff -u file1.txt file2.txt | nl
```

### 準備編輯
```bash
nl -b a file.txt | grep "^[ ]*[0-9]*[ ]*pattern"
```

## 空行處理

```rust
let is_empty = line.trim().is_empty();
let should_number = if number_all {
    true
} else {
    !is_empty  // 預設不編號空行
};
```

預設只編號非空行。

## 底層系統呼叫

`nl` 使用：
- `read()`：讀取輸入
- `write()`：寫入輸出

## 實用範例

```bash
# 基本行號
nl file.txt

# 編號所有行
nl -b a file.txt

# 自訂格式
nl -n rz -s " | " file.txt
# 輸出：
# 000001 | line1
# 000002 | line2
```

## 與 cat -n 的差異

| 特性 | `nl` | `cat -n` |
|------|------|----------|
| 空行行號 | 空白 | 有（遞增）|
| 格式化 | 多種 | 簡單 |
| 可配置 | 高 | 低 |

## 相關指令

- `cat -n`：顯示行號
- `wc -l`：行數統計
- `head`：開頭行