# std::path

跨平台路徑處理。

## Path 和 PathBuf

```rust
use std::path::Path;
use std::path::PathBuf;

let path = Path::new("/home/user/file.txt");
let mut buf = PathBuf::new();
buf.push("/home");
buf.push("user");
```

- `Path`：不可變的 borrowed 參考
- `PathBuf`：可變的 owned 字串

## 路徑組成

```rust
use std::path::Path;

let path = Path::new("/home/user/projects/main.rs");

path.file_name()     // Some("main.rs")
path.extension()     // Some("rs")
path.parent()        // Some("/home/user/projects")
path.is_absolute()   // true
path.is_relative()   // false
```

## 組合路徑

```rust
use std::path::Path;

let base = Path::new("/home/user");
let file = base.join("projects").join("main.rs");
// "/home/user/projects/main.rs"
```

## 查詢

```rust
let path = Path::new("/tmp/data/file.txt");

path.exists()        // true
path.is_file()       // true
path.is_dir()        // false
path.is_symlink()    // false
```

## to_string_lossy

```rust
let path = Path::new("/tmp/檔案.txt");
let s = path.to_string_lossy();
```

將路徑轉為字串，處理無效 UTF-8。

## as_os_str

```rust
let path = Path::new("/tmp/test");
let os_str = path.as_os_str();
```

取得 `&OsStr`，用於與作業系統 API 交互。

## components

```rust
use std::path::Path;

let path = Path::new("/home/user/file.txt");
for component in path.components() {
    println!("{:?}", component);
}
// Parent, Parent, RootDir, Normal("home"), Normal("user"), Normal("file.txt")
```

## starts_with / ends_with

```rust
let path = Path::new("/home/user/file.txt");

path.starts_with("/home")        // true
path.ends_with("file.txt")       // true
```

## 檔案名和副檔名

```rust
let path = Path::new("/home/user.tar.gz");

path.file_name()     // Some("user.tar.gz")
path.extension()      // Some("gz")
path.file_stem()      // Some("user.tar")
```

## 本專案使用

```rust
// cp.rs
use std::path::Path;

let src = Path::new(&args[1]);
let dst = Path::new(&args[2]);
if src.is_dir() {
    // ...
}

// dirname.rs
let path = Path::new(&args[1]);
let parent = path.parent();

// ls.rs
for entry in fs::read_dir(Path::new(&path))? {
    let name = entry.file_name();
    // ...
}
```

## Path 與 String

```rust
let path_str = "/tmp/test";

// String → Path
let path = Path::new(path_str);

// Path → String（可能失敗）
let s = path.to_string_lossy();  // 總是成功
// path.to_str()                  // 可能失敗
```

## 底層機制

- **Linux/macOS**：使用作業系統的路徑表示
- **xv8**：遵循 POSIX 路徑語義

## POSIX 路徑語法

| 語法 | 意義 |
|------|------|
| `/` | 根目錄 |
| `.` | 目前目錄 |
| `..` | 父目錄 |
| `~` | 家目錄（需 shell 擴展）|

## 跨平台考量

- Windows 使用 `\` 分隔，Rust Path 處理
- Unix 使用 `/` 分隔
- xv8 遵循 Unix 語法

## 相關模組

- `std::fs`：檔案操作
- `std::env`：環境變數（如 HOME）