# mkdir — 建立目錄

`mkdir`（make directory）用於建立一個或多個新目錄。

## 核心功能

`mkdir` 的實作非常簡潔，主要調用兩個 Rust 標準庫函式：

```rust
fs::create_dir(path)      // 建立單一目錄
fs::create_dir_all(path)  // 建立包含所有父目錄的完整路徑
```

## 選項處理

```rust
let mut parents = false;      // -p：建立父目錄鏈
let mut mode: Option<u32> = None;  // -m：設定許可權模式
```

`-p` 選項允許建立嵌套目錄，例如 `mkdir -p a/b/c` 即使 `a` 和 `b` 不存在也會建立。

## 模式設定

```rust
if let Some(m) = mode {
    let _ = fs::set_permissions(path, std::fs::Permissions::from_mode(m & 0o777));
}
```

許可權使用八進位字串解析：
```rust
u32::from_str_radix(&args[i], 8).unwrap_or(0o777)
```

預設許可權是 0o777（所有權限），但會被 process 的 umask 影響。

## 底層系統呼叫

- `mkdir(path, mode)`：在 POSIX 中這是一個獨立的系統呼叫
- `umask`：決定新檔案/目錄的預設許可權（在 shell 中設定）
- `chmod(path, mode)`：設定許可權（如有指定）

## umask 的作用

umask（user file creation mask）用於遮罩新建立的檔案/目錄的許可權位。例如 umask 為 022 時：
- 建立的目錄許可權 = 0o777 & ~0o022 = 0o755

## 錯誤處理

常見錯誤：
- 目錄已存在：`EEXIST`
- 父目錄不存在且無 `-p`：`ENOENT`
- 路徑中存在非目錄的檔案：`ENOTDIR`
- 無寫入父目錄的許可權：`EACCES`

## 與其他工具的比較

- `mkdir -p`：安全建立嵌套目錄，不會因已存在而失敗
- `install -d`：可以同時設定擁有者和許可權

## 相關指令

- `rmdir`：刪除空目錄
- `ls -ld`：檢查目錄是否存在及其許可權