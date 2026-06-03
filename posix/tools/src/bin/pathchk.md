# pathchk — 檢查路徑名稱有效性

`pathchk` 檢查檔案路徑名稱是否符合 POSIX 規範。

## 核心設計

```rust
fn check_path(path: &str) -> bool {
    if path.is_empty() {
        eprintln!("pathchk: empty pathname");
        return false;
    }
    if path.len() > 255 {
        eprintln!("pathchk: {}: pathname too long (max 255)", path);
        return false;
    }
    // ... 檢查每個路徑元件
}
```

檢查三個條件：
1. 路徑不為空
2. 整體路徑長度 ≤ 255
3. 每個路徑元件（目錄/檔案）長度 ≤ 255

## POSIX 路徑限制

| 限制 | 值 |
|------|-----|
| 路徑名最大長度 | 255 位元組（POSIX定義為 `NAME_MAX`）|
| 路徑總長度 | 實作定義（通常 4096）|

## 目錄名稱檢查

```rust
if let Some(dirname) = parent.file_name() {
    let d = dirname.to_string_lossy();
    if d.len() > 255 {
        eprintln!("pathchk: {}: pathname too long", path);
        return false;
    }
}
```

## 檔案名稱檢查

```rust
if let Some(filename) = p.file_name() {
    let f = filename.to_string_lossy();
    if f.len() > 255 { ... }
    if f.contains('/') { ... }  // 不允許 /
}
```

## 典型用途

### 驗證使用者輸入
```bash
if ! pathchk "$user_path"; then
    echo "Invalid path"
    exit 1
fi
```

### 批次檢查多個路徑
```bash
for path in "${paths[@]}"; do
    pathchk "$path" || echo "Invalid: $path"
done
```

## 選項

完整 `pathchk` 支援：
- `-p`：檢查較舊的 POSIX 路徑限制（255位元組）
- `-P`：檢查連字號開頭
- `--portability`：使用較嚴格的檢查

## 與系統限制

`pathchk` 只做靜態檢查，不嘗試存取檔案系統。

```bash
pathchk /very/long/path/that/exceeds/limits
# 即使路徑不存在也會報錯
```

## 父親路徑長度

```rust
let parent_str = parent.to_string_lossy();
if parent_str.len() > 255 {
    eprintln!("pathchk: {}: pathname too long", path);
    return false;
}
```

## 為何需要 pathchk？

### 跨平台相容性
- 不同的檔案系統有不同的命名限制
- `pathchk` 在寫入前驗證路徑有效性

### 避免未來問題
- 看似合法的路徑可能在某些系統失敗
- 提早檢查可避免執行時錯誤

## 安全性

`pathchk` 只讀取路徑字串，不執行任何系統呼叫，是安全的工具。

## 底層原理

`pathchk` 依賴 `Path::file_name()` 和 `Path::parent()` 解析路徑元件。

## 實用範例

```bash
# 基本檢查
pathchk /home/user/file.txt

# 檢查多個路徑
pathchk path1 path2 path3

# 在腳本中使用
if pathchk "$INPUT_PATH"; then
    cp source "$INPUT_PATH"
fi
```

## 與 readlink 的比較

- `pathchk`：驗證路徑格式
- `readlink`：解析符號連結

## 相關指令

- `readlink`：讀取符號連結
- `realpath`：解析完整路徑
- `test`：測試檔案屬性