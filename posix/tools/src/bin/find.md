# find — 搜尋檔案

`find` 是 Unix 系統中最強大的檔案搜尋工具，能在目錄樹中遞迴搜尋符合條件的檔案。

## 核心設計

`find` 採用遞迴走訪策略：

```rust
fn find_dir(path: &Path, opts: &FindOpts) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            // 檢查是否符合條件
            if matched {
                println!("{}", p.display());
            }
            // 如果是目錄，遞迴進入
            if p.is_dir() && !entry.file_type().map(|x| x.is_symlink()).unwrap_or(false) {
                find_dir(&p, opts);
            }
        }
    }
}
```

## 條件匹配

```rust
let mut matched = true;
if let Some(pattern) = &opts.name {
    if !name.contains(pattern) { matched = false; }
}
if let Some(t) = opts.type_filter {
    let ft = entry.file_type().ok();
    let is_match = match t {
        b'f' => ft.map(|x| x.is_file()).unwrap_or(false),  // 普通檔案
        b'd' => ft.map(|x| x.is_dir()).unwrap_or(false),   // 目錄
        b'l' => ft.map(|x| x.is_symlink()).unwrap_or(false), // 符號連結
        _ => true,
    };
    if !is_match { matched = false; }
}
```

注意：符號連結預設會遞迴進入，除非明確排除。

## 選項解析

```rust
match args[i].as_str() {
    "-name" => { opts.name = Some(args[i + 1].clone()); }
    "-type" => { opts.type_filter = args[i + 1].as_bytes().first().copied(); }
}
```

目前 xv8 的 `find` 支援：
- `-name pattern`：檔案名包含 pattern
- `-type [fdl]`：檔案類型（file/dir/link）

## 預設路徑

如果沒有指定搜尋路徑，預設為目前目錄：

```rust
let paths: Vec<&str> = if i < args.len() { args[i..].iter().map(String::as_str).collect() } else { vec!["."] };
```

## 與其他工具的結合

`find` 真正的威力在於與 `-exec` 或 `xargs` 結合：

```bash
# 刪除所有 .tmp 檔案
find . -name "*.tmp" -exec rm {} \;

# 找出的檔案交給其他命令處理
find . -name "*.c" | xargs grep "TODO"
```

xv8 的 `find` 目前不支援 `-exec`，但可以透過管道：

```bash
find . -name "*.c" | xargs grep "TODO"
```

## 條件組合

完整的 `find` 支援多條件：
- `-and` / `-or`：邏輯與/或
- `-not` / `!`：邏輯非
- `-path`：路徑匹配

```bash
find . -name "*.c" -and -mtime -7    # 7天內修改的 .c 檔案
```

## 時間條件

`find` 經典的時間條件：
- `-mtime n`：修改時間 n 天
- `-atime n`：訪問時間 n 天
- `-ctime n`：inode 變更時間 n 天
- `-newer file`：比指定檔案更新

```bash
find . -mtime +7     # 7天前修改的
find . -mtime -7     # 7天內修改的
find . -mtime 7      # 正好第7天
```

## 深度控制

- `-maxdepth n`：最多深入 n 層
- `-mindepth n`：至少深入 n 層

```bash
find . -maxdepth 2 -name "*.c"  # 只在目前目錄和一層子目錄中搜尋
```

## 輸出格式化

- `-print`（預設）：輸出路徑
- `-print0`：以 null 字元分隔（處理特殊字元）
- `-printf`：格式化輸出

## 底層系統呼叫

`find` 使用的主要 syscall：
- `getdents64(fd, buf, size)`：讀取目錄項目
- `lstat(path, buf)`：獲取檔案狀態（不追隨符號連結）
- `stat(path, buf)`：獲取檔案狀態（追隨符號連結）

## 安全性考量

`find` 可能接觸大量檔案，注意：
- 避免在網路檔案系統上大量搜尋
- 使用 `-prune` 排除不需搜尋的目錄

## 效能最佳化

- 先用 `-maxdepth` 限制搜尋範圍
- 使用 `-name` 而非 `-path`（可能更快）
- 避免過度使用 `-o`（or）條件

## 典型用途

```bash
# 找所有 .log 檔案
find /var/log -name "*.log"

# 找空目錄
find . -type d -empty

# 找可執行檔
find . -type f -executable

# 找 777 許可權的檔案
find . -type f -perm 777
```

## 與 locate 的比較

- `find`：即時搜尋，每次都走訪目錄樹
- `locate`：基於資料庫，搜尋快速但需要定期更新

## 相關指令

- `xargs`：將 find 結果作為參數傳給其他命令
- `exec`：在 find 中直接執行命令
- `locate`：基於資料庫的快速搜尋
- `fd`：Rust 實現的更快的 find 替代