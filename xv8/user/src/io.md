# io — I/O  Trait 與巨集

提供類似 `std::io` 的 Read/Write trait，以及 print/println 巨集。

## Read Trait

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SysError>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), SysError> {
        while !buf.is_empty() {
            let n = self.read(buf)?;
            if n == 0 {
                return Err(SysError::IoError);
            }
            buf = &mut buf[n..];
        }
        Ok(())
    }
}
```

### Fd 實作

```rust
impl Read for Fd {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SysError> {
        syscall::read(*self, buf)
    }
}
```

## Write Trait

```rust
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SysError>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), SysError> {
        while !buf.is_empty() {
            let n = self.write(buf)?;
            buf = &buf[n..];
        }
        Ok(())
    }
}
```

### Fd 實作

```rust
impl Write for Fd {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SysError> {
        syscall::write(*self, buf)
    }
}
```

## 標準串流

```rust
pub struct Stdin;
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SysError> {
        syscall::read(Fd::STDIN, buf)
    }
}

pub struct Stdout;
impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SysError> {
        syscall::write(Fd::STDOUT, buf)
    }
}
impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

pub struct Stderr;
impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SysError> {
        syscall::write(Fd::STDERR, buf)
    }
}
```

## 列印巨集

```rust
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        <$crate::Stdout as core::fmt::Write>::write_fmt(
            &mut $crate::Stdout,
            format_args!($($arg)*),
        ).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
```

## 使用範例

```rust
print!("Value: {}", 42);           // 無換行
println!("Hello, world!");          // 有換行
println!("{} + {} = {}", 1, 2, 3);  // 格式化
eprintln!("Error: {}", err);        // 錯誤輸出

// 使用 trait
let mut fd = open("file.txt", OpenFlag::READ_ONLY)?;
let mut buf = [0u8; 512];
fd.read(&mut buf)?;
// 或
buf.read_exact(&mut fd)?;  // 從 stdin 讀取
```

## 與 fmt 的整合

`core::fmt::Write` 由 `println!` 和 `print!` 內部使用：
- `format_args!` 創建 `Arguments`
- 傳遞到 `Stdout::write_fmt`
- `write_str` 將格式化後的字串寫入

## 錯誤處理

- `read` 失敗：回傳 `SysError`
- `write` 失敗：回傳 `SysError`
- 巨集使用 `.unwrap()`，失敗時 panic

## 相關主題

- [[syscall]]：底層系統呼叫