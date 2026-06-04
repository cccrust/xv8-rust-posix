# comm — 逐行比較兩個排序過的檔案

`comm`（common）比較兩個已經排序的檔案，輸出三欄結果：只在第一個檔案中的行、只在第二個中的行、兩者皆有的行。

## 核心設計

```rust
let lines1 = read_lines(file1);
let lines2 = read_lines(file2);

let mut i = 0usize;
let mut j = 0usize;

while i < lines1.len() || j < lines2.len() {
    if i >= lines1.len() {
        println!("\t\t{}", lines2[j]);  // 只有第二個
        j += 1;
    } else if j >= lines2.len() {
        println!("{}", lines1[i]);  // 只有第一個
        i += 1;
    } else {
        match lines1[i].cmp(&lines2[j]) {
            Ordering::Less => {
                println!("{}", lines1[i]);  // 只有第一個
                i += 1;
            }
            Ordering::Greater => {
                println!("\t\t{}", lines2[j]);  // 只有第二個
                j += 1;
            }
            Ordering::Equal => {
                println!("\t{}", lines1[i]);  // 兩者皆有
                i += 1;
                j += 1;
            }
        }
    }
}
```

## 三欄輸出

`comm` 的輸出分為三欄，用 Tab 分隔：

```
第1欄：只在檔案1中的行
第2欄：只在檔案2中的行
第3欄：兩檔案皆有的行
```

### 範例

檔案1（a.txt）：
```
apple
banana
cherry
```

檔案2（b.txt）：
```
banana
cherry
date
```

執行 `comm a.txt b.txt`：
```
apple
		banana
		cherry
	date
```

- `apple`：只在 a.txt
- `banana`：兩者皆有
- `cherry`：兩者皆有
- `date`：只在 b.txt

## 選項：欄位抑制

```rust
match c {
    '1' | '2' | '3' => {} // column suppression
}
```

- `-1`：抑制第 1 欄（只在檔案 1）
- `-2`：抑制第 2 欄（只在檔案 2）
- `-3`：抑制第 3 欄（兩者皆有）

```bash
comm -12 a.txt b.txt   # 只顯示兩者皆有的行
comm -23 a.txt b.txt   # 只顯示只在 b.txt 的行
```

## 輸入來源

```rust
fn read_lines(name: &str) -> Vec<String> {
    if name == "-" {
        io::stdin().lock().lines().map(|l| l.unwrap_or_default()).collect()
    } else {
        std::fs::read_to_string(Path::new(name))
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .collect()
    }
}
```

支援檔案或 stdin（`-` 表示）。

## 預排序要求

`comm` 要求輸入**已經排序過**。如果未排序，結果會不正確。

## 與 diff 的比較

| 特性 | `comm` | `diff` |
|------|---------|--------|
| 輸出 | 三欄 | unified/side-by-side |
| 排序 | 必須預排序 | 自動處理 |
| 用途 | 集合運算 | 詳細差異 |

## 典型用途

### 找兩檔案的共同行
```bash
comm -12 file1.txt file2.txt
```

### 找只在檔案1中的行
```bash
comm -23 file1.txt file2.txt | head -1  # 或用 comm -13
```

### 集合差異
```bash
# 只在 A 不在 B
comm -23 A B

# 只在 B 不在 A
comm -13 A B

# 兩者皆有的行
comm -12 A B
```

### 配合 sort 使用
```bash
comm -12 <(sort file1.txt) <(sort file2.txt)
```

## 輸出格式

預設輸出中：
- 第 1 欄：無縮排
- 第 2、3 欄：以 Tab 縮排

```bash
line only in file1
	line only in file2
	line in both
```

## 空白行處理

`comm` 將空白行視為普通行，可能導致對齊問題。

## 底層系統呼叫

`comm` 主要依賴：
- `read()`：讀取檔案
- 記憶體內排序

## 效能考量

`comm` 需要將兩個檔案全部讀入記憶體，複雜度 O(n)。

## 實用範例

```bash
# 找共同行（交集）
comm -12 sorted1.txt sorted2.txt

# 找差異行
comm -13 file1 file2  # 只在 file1
comm -23 file1 file2  # 只在 file2

# 結合 set 運算
comm -23 <(sort -u set1) <(sort -u set2) | wc -l
```

## 與其他工具的結合

```bash
# 找重複行
comm -12 <(sort file1) <(sort file2)

# 統計共同行數
comm -12 file1 file2 | wc -l
```

## 相關指令

- `diff`：詳細差異比較
- `diff3`：三方比較
- `join`：基於欄位連接
- `uniq`：去除相鄰重複行