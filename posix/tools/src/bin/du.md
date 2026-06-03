# du — 磁碟使用量

`du`（disk usage）顯示每個檔案和目錄佔用的磁碟空間。

## 核心設計

```rust
fn du_dir(path: &Path, depth: usize, max_depth: Option<usize>, human: bool, summarize: bool) -> u64 {
    if let Some(md) = max_depth {
        if depth > md { return 0; }
    }

    let mut total = 0u64;
    let is_dir = path.is_dir();

    if is_dir {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                total += du_dir(&entry.path(), depth + 1, max_depth, human, summarize);
            }
        }
    }

    if let Ok(meta) = path.metadata() {
        total += meta.len();
    }

    if depth == 0 || (!summarize && depth > 0) || (summarize && depth == 0) {
        if human {
            println!("{}\t{}", format_size(total), path.display());
        } else {
            println!("{}\t{}", total, path.display());
        }
    }

    total
}
```

`du` 遞迴計算目錄樹中所有檔案的大小總和。

## 遞迴計算

```rust
if is_dir {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            total += du_dir(&entry.path(), depth + 1, max_depth, human, summarize);
        }
    }
}
```

對每個子目錄遞迴呼叫，最後加總。

## 選項處理

```rust
let mut summarize = false;  // -s：只顯示總計
let mut human = false;       // -h：人類可讀格式
let mut max_depth: Option<usize> = None;  // -d N：最大深度
```

- `-s`（summarize）：只顯示每個參數的總計，不顯示子目錄
- `-h`（human）：以 K、M、G 等單位顯示
- `-d N`：限制顯示深度

## 人類可讀格式

```rust
fn format_size(bytes: u64) -> String {
    const UNITS: &[char] = &['K', 'M', 'G', 'T'];
    let mut size = bytes as f64;
    for &unit in UNITS {
        if size >= 1024.0 {
            size /= 1024.0;
            if size < 1024.0 {
                return format!("{:.1}{}", size, unit);
            }
        }
    }
    format!("{:.1}P", size)
}
```

遞進除以 1024，找出最適合的單位。

## 深度控制

```rust
if let Some(md) = max_depth {
    if depth > md { return 0; }
}
```

`depth > max_depth` 時直接返回 0，不輸出也不遞迴。

## 輸出條件

```rust
if depth == 0 || (!summarize && depth > 0) || (summarize && depth == 0) {
    // 輸出
}
```

- `depth == 0`：根目錄（最頂層）
- `!summarize && depth > 0`：非匯總模式且有深度
- `summarize && depth == 0`：匯總模式（只輸出總計）

## 預設行為

不帶參數時，`du` 顯示目前目錄下所有項目的大小：

```bash
du
```

## 與 df 的比較

| 特性 | `du` | `df` |
|------|------|------|
| 計算方式 | 實際檔案大小 | 檔案系統級別 |
| 用途 | 分析目錄佔用 | 磁碟空間統計 |
| 速度 | 較慢（需掃描） | 較快（讀 superblock） |

## 實際應用

### 找出最大的目錄
```bash
du -sh * | sort -h
```

### 只顯示總計
```bash
du -sh /var/log
```

### 限制深度
```bash
du -h -d 1 /home
```

### 排除特定模式
```bash
du --exclude='*.log' /path
```

## inode 佔用

`du` 只計算檔案資料的大小，不計算：
- 檔案系統中繼資料
- 保留空間
- 已刪除但仍開啟的檔案

## 符號連結

`du` 預設會跟隨符號連結（使用 `-L` 避免）。

## 效能考量

- `du` 需要掃描整個目錄樹，時間複雜度 O(n)
- 大目錄可能很慢
- 使用 `--exclude` 可減少處理

## 底層系統呼叫

- `readdir`：列舉目錄項目
- `stat/lstat`：獲取檔案大小

## 輸出格式

```
4096    ./subdir
8192    ./file.txt
12288   .
```

第一欄是大小（位元組），第二欄是路徑。

## 典型用途

```bash
# 目前目錄的總大小
du -sh .

# 所有子目錄的大小（排序）
du -h * | sort -h

# 前 10 大的目錄
du -h /var 2>/dev/null | sort -rh | head -10
```

## 相關指令

- `df`：顯示檔案系統空間
- `ncdu`：互動式磁碟使用分析
- `duf`：現代化的 `du` 替代