# std::io

Rust 的輸入輸出模組，本專案的 POSIX 工具大量使用。

## 核心 traits

### Read

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> { ... }
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> { ... }
}
```

從來源讀取位元組。

### Write

```rust
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}
```

寫入位元組。

### BufRead

```rust
pub trait BufRead: Read {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
    fn lines(&mut self) -> Lines<Self>;
}
```

帶緩衝的讀取，支援逐行處理。

## 緩衝 I/O

```rust
let file = File::open("file.txt")?;
let reader = BufReader::new(file);
let mut line = String::new();
reader.read_line(&mut line)?;
```

xv8 中的使用：
```rust
let reader = std::io::BufReader::new(file);
for line in reader.lines() {
    println!("{}", line?);
}
```

## stdin/stdout

```rust
use std::io::{stdin, stdout, Write};

let input = stdin().lock().read_line(&mut line)?;
stdout().write_all(b"Hello\n")?;
```

xv8 的實現掛鉤到主控台。

## Error 處理

```rust
use std::io;

fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
```

## 本專案使用

xv8 的 POSIX 工具幾乎都依賴 std::io：

```rust
// cat.rs
use std::io::{self, Read, Write};
let mut buffer = [0u8; 8192];
io::stdin().lock().read(&mut buffer)?;

// echo.rs
io::stdout().write_all(args.join(" ").as_bytes())?;
```

## Copy

```rust
pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where R: Read, W: Write;
```

複製資料，xv8 工具中的實現：
```rust
std::io::copy(&mut input, &mut output)?;
```

## Lines

```rust
let reader = BufReader::new(file);
for line in reader.lines() {
    let line = line?;
    // 處理行
}
```

## 與作業系統的關係

std::io 在不同平台上：
- **Linux/macOS**：調用 libc read/write
- **xv8 RISC-V**：通過 syscall 包裝

## 底層機制

```ignore
Rust Code → std::io → libc/syscall → Kernel → Hardware
```

## 相關模組

- `std::fs`：檔案系統操作
- `std::env`：環境變數
- `std::process`：程序管理