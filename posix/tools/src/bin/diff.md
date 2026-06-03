# diff — 比較兩個檔案的差異

`diff` 用於比較兩個檔案的內容差異，輸出它們之間的不同之處。

## 核心演算法：LCS

`diff` 使用「最長公共子序列」（Longest Common Subsequence）演算法：

```rust
let ops = lcs_diff(&lines1, &lines2);

enum DiffOp {
    Same(&'static str),   // 兩邊相同
    Delete(&'static str),  // 僅左邊有（被刪除）
    Insert(&'static str), // 僅右邊有（被新增）
}
```

LCS 問題的本質：
- 找出兩個序列中最長的共同子序列（保持相對順序）
- 不是連續子序列，可以跳過中間的元素

## 輸出格式

`diff` 輸出稱為「unified diff」格式：

```bash
--- file1.txt
+++ file2.txt
@@ -a,b +c,d @@
 context line
-delete line
+insert line
```

### 行前綴含義

- `空格`（space）：兩邊相同的上下文
- `-`（減號）：只在第一個檔案中（被刪除）
- `+`（加號）：只在第二個檔案中（被新增）

## Hunk 合併

`diff` 將相關的變化組織成「hunks」：

```rust
// 找到變化區域的開始
let mut hunk_start = pos;
let mut ctx = 0;
while hunk_start > 0 && ctx < 3 {
    hunk_start -= 1;
    match &ops[hunk_start] {
        DiffOp::Same(_) => ctx += 1,
        _ => { ctx = 0; }
    }
}
```

每個 hunk 包含：
- 最多 3 行上下文
- 變化的具體行
- `@@` 行說明位置

## 行號計算

```rust
fn compute_a_line(ops: &[DiffOp], pos: usize, lines: &[&str]) -> usize {
    let mut line = 1;
    for i in 0..pos {
        if matches!(&ops[i], DiffOp::Same(_)) { line += 1; }
        else if matches!(&ops[i], DiffOp::Delete(_)) { line += 1; }
    }
    line
}
```

計算受影響的行號範圍。

## diff 的用途

### 版本控制
```bash
# 比較檔案
diff old.c new.c

# 產生補丁
diff -u old.c new.c > changes.patch
```

### 腳本中的條件判斷
```bash
if diff -q file1.txt file2.txt > /dev/null; then
    echo "Files are identical"
fi
```

### 自動化測試
```bash
# 比較輸出
./program > expected.txt
./program > actual.txt
diff expected.txt actual.txt
```

## 演算法複雜度

標準 LCS 演算法是 O(n*m) 的時間和空間複雜度，其中 n 和 m 是兩個檔案的行數。

對於大檔案，有更優化的演算法（如 Myers 算法），用於 GNU diffutils。

## 輸出選項

完整的 `diff` 支援多種格式：

- `-u`：unified format
- `-c`：context format
- `-e`：ed script format
- `-n`：rcs format
- `-y`：side-by-side

## 與其他工具的比較

| 工具 | 特點 |
|------|------|
| `diff` | 行級比較 |
| `cmp` | 位元組級比較（更快） |
| `comm` | 逐行比較，輸出三欄 |

## 實際應用

### 補丁應用
```bash
# 產生補丁
diff -u original.c modified.c > fix.patch

# 應用補丁
patch original.c < fix.patch
```

### 目錄比較
```bash
diff -r dir1/ dir2/
```

### 忽略空白
```bash
diff -w file1.txt file2.txt
```

## 底層系統呼叫

`diff` 主要依賴：
- `open/read/close`：讀取檔案
- `readdir`：目錄比較時列舉

## 效能考量

- `diff` 需要將整個檔案讀入記憶體
- 對於大檔案，考慮使用 `cmp`（只比較到第一個不同點）
- `-q`（brief）選項可以在找到第一個差異後停止

## 相關指令

- `patch`：應用補丁
- `cmp`：位元組級比較
- `comm`：三欄輸出
- `sdiff`：side-by-side 比較
- `diff3`：三方比較