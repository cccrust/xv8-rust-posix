# unexpand — Space 轉換為 Tab

`unexpand` 將行內的空格序列轉換為 Tab。

## 核心設計

```rust
while j < chars.len() {
    if chars[j] == ' ' {
        let mut space_count = 0;
        while j + space_count < chars.len() && chars[j + space_count] == ' ' {
            space_count += 1;
        }
        let next_tab = tabs - (col % tabs);
        if space_count >= next_tab {
            out.push('\t');
            col += next_tab;
            j += next_tab;
        } else {
            // 空格不夠，保留
        }
    }
}
```

核心邏輯：計算到下一個 Tab stop 的距離，夠了就替換。

## Tab Stop

```rust
let next_tab = tabs - (col % tabs);
```

Tab stop 是 `tabs` 的倍數位置。

## 轉換條件

```bash
# 位置        012345678901234567890
# 輸入：      "      Hello"  (6 spaces)
# Tab stop:   |----8----|    (8)
# 轉換：      \tHello    (1 tab + 剩餘)
```

轉換條件：連續空格 ≥ 到下一個 Tab stop 的距離。

## 與 expand 的關係

| 工具 | 轉換 |
|------|------|
| `expand` | Tab → Space |
| `unexpand` | Space → Tab |

## 典型用途

### 減少檔案大小
```bash
unexpand file.txt > compressed.txt
```

### 原始文字處理
```bash
# Makefiles 需要 Tab
unexpand Makefile.in > Makefile
```

## 選項

```rust
match c {
    'a' => {} // all (default)
    't' => { i += 1; if i < args.len() { tabs = args[i].parse().unwrap_or(8); } }
}
```

- `-a`：轉換所有可轉換的空格（預設）
- `-t N`：設定 Tab 寬度為 N

## 邊界情況

### 連續空格
```
"  " (2 spaces at col 0) → \t if tabs=8
```

### 不連續空格
```
"a b" → 保留（不在同一 Tab stop 範圍）
```

## 欄位追蹤

```rust
let mut col = 0;

out.push('\t');
col += next_tab;  // Tab 移動到下一個 stop

out.push(' ');
col += 1;         // Space 只移動 1
```

## 與空白字元的關係

`unexpand` 只處理普通空格，不處理：
- `\t`（已是 Tab）
- `\n`（換行）

## 底層系統呼叫

`unexpand` 使用：
- `read()`：讀取輸入
- `write()`：寫入輸出

## 實用範例

```bash
# 基本用法
unexpand file.txt

# 指定 Tab 寬度
unexpand -t 4 file.txt

# 配合 expand（可逆）
expand file.txt | unexpand -t 8
```

## 轉換示意

```
原始：        "Hello    World"
             ├────┬────┤
             6 spaces (位置 5-10)

Tab stop 8：  "Hello\tWorld"
             ├─┘
             轉換 6 spaces → \t（因為 6 ≥ 3 到 stop 8）
```

## 注意事項

`unexpand` 可能改變二進制資料，應只用於文字檔案。

## 相關指令

- `expand`：Tab → Space
- `tr`：字元轉換
- `sed`：-stream editor