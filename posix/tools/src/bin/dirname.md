# dirname — 去除路徑中的檔案名

`dirname` 從完整路徑中提取目錄部分（去除最後的檔案名）。

## 核心設計

```rust
let parent = if args[1] == "/" || args[1] == "//" {
    args[1].clone()
} else {
    match path.parent() {
        Some(p) if p.as_os_str().is_empty() => ".".to_string(),
        Some(p) => p.to_string_lossy().to_string(),
        None => ".".to_string(),
    }
};
```

使用 `Path::parent()` 取得父目錄。

## 路徑結構

```
/home/user/projects/main.rs
└─┬─┘ └────┘ └──┘
  │     │      └─── basename
  │     └─────────── dirname
  └───────────────── root
```

- **dirname**：`/home/user/projects`（目錄部分）
- **basename**：`main.rs`（檔案名部分）

## POSIX 規範

POSIX 明確定義了 `dirname` 的行為：

| 輸入 | 輸出 |
|------|------|
| `/` | `/` |
| `//` | `//` |
| `/usr` | `/` |
| `foo` | `.` |
| `foo/bar` | `foo` |

## 邊界情況

### 純根路徑
```bash
dirname /
# 輸出：/

dirname //
# 輸出：//
```

### 相對路徑
```bash
dirname file.txt
# 輸出：.

dirname ./path/to/file
# 輸出：./path/to
```

### 多重 /
```bash
dirname /usr/local/bin
# 輸出：/usr/local
```

## 與 basename 的配合

```bash
path="/home/user/file.txt"
dirname "$path"    # /home/user
basename "$path"    # file.txt

# 組合使用
dir=$(dirname "$path")
file=$(basename "$path")
```

## 典型用途

### Shell 腳本
```bash
# 取得腳本所在目錄
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# 移動檔案到同目錄
mv /path/to/file.txt "$(dirname /path/to/file.txt)/backup.txt"
```

### 檔案操作
```bash
# 複製到同目錄
cp /source/path/file.txt "$(dirname /source/path/file.txt)/copy.txt"
```

## 路徑解析

`dirname` 只是字串操作，不需要路徑實際存在：
```bash
dirname /nonexistent/path/file.txt
# 輸出：/nonexistent/path
```

## 與 realpath 的比較

- `dirname`：簡單的字串處理
- `realpath`：解析符號連結並返回規範路徑

## 安全性

`dirname` 不執行任何操作，是安全的工具。

## 底層系統呼叫

`dirname` 不依賴系統呼叫，只是路徑字串處理。

## 實用範例

```bash
# 取得目前目錄
pwd | dirname

# 批次取得目錄
for f in /path/to/*.txt; do
    dir=$(dirname "$f")
    echo "File $f is in directory $dir"
done

# 配合 cd
cd $(dirname "$0")
```

## bash 內建

某些 shell（如 bash）提供 `dirname` 作為內建命令，但也有 `/usr/bin/dirname` 外部程式。

## 相關指令

- `basename`：提取檔案名
- `realpath`：規範化路徑
- `readlink`：讀取符號連結目標