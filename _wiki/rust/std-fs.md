# std::fs

Rust 的檔案系統操作模組。

## 主要函式

### File::open

```rust
use std::fs::File;

let mut file = File::open("input.txt")?;
```

以唯讀方式開啟檔案。

### File::create

```rust
let mut file = File::create("output.txt")?;
```

創建或截斷檔案。

### OpenOptions

```rust
use std::fs::OpenOptions;

let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("file.txt")?;
```

精確控制開啟模式。

## 讀寫

```rust
use std::io::{Read, Write};

let mut file = File::open("input.txt")?;
let mut contents = String::new();
file.read_to_string(&mut contents)?;

let mut file = File::create("output.txt")?;
file.write_all(b"Hello, world!")?;
```

## 目錄操作

### read_dir

```rust
use std::fs;

for entry in fs::read_dir("/path/to/dir")? {
    let entry = entry?;
    println!("{:?}", entry.file_name());
}
```

### create_dir / create_dir_all

```rust
fs::create_dir("/tmp/newdir")?;
fs::create_dir_all("/path/to/nested/dir")?;
```

## metadata

```rust
use std::fs;

let metadata = fs::metadata("file.txt")?;
println!("Size: {} bytes", metadata.len());
println!("Is file: {}", metadata.is_file());
println!("Is dir: {}", metadata.is_dir());
```

## copy / rename / remove

```rust
fs::copy("source.txt", "dest.txt")?;
fs::rename("old.txt", "new.txt")?;
fs::remove_file("temp.txt")?;
fs::remove_dir("emptydir")?;
```

## 本專案使用

```rust
// cp.rs
use std::fs;

fs::copy(&source, &destination)?;

// ls.rs
for entry in fs::read_dir(path)? {
    let metadata = entry?.metadata()?;
    // ...
}

// tar.rs
let data = fs::read(path)?;
fs::write(name, file_data)?;
```

## 錯誤處理

```rust
use std::io;

match File::open("nonexistent") {
    Ok(file) => { /* 成功 */ }
    Err(e) => eprintln!("Error: {}", e),
}
```

## 與 std::io 的關係

`std::fs::File` 實現了 `Read` 和 `Write` traits，可以直接用 std::io 的工具處理。

```rust
use std::io::BufReader;

let file = File::open("file.txt")?;
let reader = BufReader::new(file);
for line in reader.lines() { /* ... */ }
```

## 底層機制

- **Linux/macOS**：調用 libc open/read/write
- **xv8**：通過 fs syscall

## 安全性考量

- 路徑遍歷：使用 `std::fs::canonicalize` 驗證路徑
- 符號連結：`metadata` vs `symlink_metadata`
- 權限檢查：取決於運行的用戶

## POSIX 對應

| Rust | POSIX |
|------|-------|
| `File::open` | `open(path, O_RDONLY)` |
| `File::create` | `open(path, O_WRONLY\|O_CREAT\|O_TRUNC)` |
| `fs::read_dir` | `opendir`/`readdir` |
| `fs::metadata` | `stat` |

## 相關模組

- `std::io`：I/O traits
- `std::path`：路徑處理