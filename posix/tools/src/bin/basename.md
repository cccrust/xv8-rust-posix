# basename — 去除路徑中的目錄部分

`basename` 從完整路徑中提取檔案或目錄名稱。

## 核心設計

```rust
let path = Path::new(&args[1]);
let mut name = match path.file_name() {
    Some(n) => n.to_string_lossy().to_string(),
    None => {
        // 處理以 / 結尾的路徑
        if args[1].ends_with('/') && args[1] != "/" {
            let trimmed = args[1].trim_end_matches('/');
            Path::new(trimmed).file_name().unwrap_or_default().to_string_lossy().to_string()
        } else {
            args[1].clone()
        }
    }
};
```

使用 `Path::file_name()` 取得檔案名稱部分。

## 路徑結構

```
/home/user/projects/main.rs
└─┬─┘ └────┘ └──┘
  │     │      └─── basename
  │     └─────────── dirname
  └───────────────── root
```

- **basename**：`main.rs`（最後一個路徑元件）
- **dirname**：`/home/user/projects`（路徑的目錄部分）

## 後綴移除

```rust
if args.len() > 2 {
    let suffix = &args[2];
    if name.ends_with(suffix) && !suffix.is_empty() {
        let end = name.len() - suffix.len();
        name.truncate(end);
    }
}
```

`basename` 的第二個參數用於移除尾隨後綴：

```bash
basename /path/to/file.txt .txt
# 輸出：file

basename /path/to/image.jpg .jpg
# 輸出：image
```

## 邊界情況

### 以 / 結尾的路徑
```bash
basename /path/to/dir/
# 輸出：dir（尾隨的 / 被忽略）
```

### 純根路徑
```bash
basename /
# 輸出：/
```

### 沒有目錄的路徑
```bash
basename filename
# 輸出：filename
```

## 典型用途

### 提取檔案名
```bash
basename /usr/local/bin/program
# 輸出：program
```

### 去除副檔名
```bash
basename /path/to/file.tar.gz .gz
# 輸出：file.tar（只移除最後一個 .gz）

# 要移除所有擴展名
basename /path/to/file.tar.gz .tar.gz
# 輸出：file
```

## 與 dirname 的配合

```bash
filename=$(basename /path/to/file.txt)
dirpath=$(dirname /path/to/file.txt)
echo "File: $filename, Dir: $dirpath"
# 輸出：File: file.txt, Dir: /path/to
```

## Shell 腳本中的用途

```bash
# 批次處理檔案
for f in *.txt; do
    name=$(basename "$f" .txt)
    echo "Processing: $name"
done
```

## 路徑解析

`basename` 不需要路徑實際存在，只做字串處理：
```bash
basename /nonexistent/path/file.txt
# 輸出：file.txt
```

## POSIX 規範

POSIX 要求 `basename`：
- 必須有兩個參數或只有一個
- 不能只使用選項

## 與其他工具的比較

| 工具 | 用途 |
|------|------|
| `basename` | 提取檔案名 |
| `dirname` | 提取目錄路徑 |
| `cut` | 按位置提取 |
| `awk` | 更靈活 |

## 安全性

`basename` 只做字串處理，不會執行任何操作，是安全的。

## 底層系統呼叫

`basename` 不依賴系統呼叫，只是字串操作。

## 實用範例

```bash
# 備份檔案
cp file.txt file.txt.bak
# 或動態
cp file.txt $(basename file.txt).bak

# 提取 URL 中的最後一部分
basename https://example.com/path/page
# 輸出：page

# 配合 for 迴圈
for url in http://example.com/a http://example.com/b; do
    fname=$(basename $url)
    curl -O $url/$fname
done
```

## dirname

`dirname` 是 `basename` 的互補：
```bash
dirname /home/user/file.txt
# 輸出：/home/user
```