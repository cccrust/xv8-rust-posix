# std::fmt

格式化輸出，是 `print!`/`println!`/`format!` 等巨集的基礎。

## Formatting traits

### Display

```rust
use std::fmt;

struct Point { x: i32, y: i32 }

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

提供 `{}` 格式化的主要方式。

### Debug

```rust
use std::fmt;

struct Point { x: i32, y: i32 }

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
```

提供 `{:?}` 和 `{:#?}` 格式化。

## format!

建立格式化的 String：

```rust
let s = format!("Hello, {}!", "world");
let x = format!("{:>10}", "right");    // "   right"
let y = format!("{:<10}", "left");     // "left      "
let z = format!("{:^10}", "center");   // "  center  "
```

## print! / println!

輸出到標準輸出：

```rust
print!("No newline");
println!("With newline");
println!("{} + {} = {}", 1, 2, 3);
```

## eprint! / eprintln!

輸出到標準錯誤：

```rust
eprint!("Error: ");
eprintln!("Something went wrong");
```

## write! / writeln!

寫入任何实现了 `Write` 的 writer：

```rust
use std::io;

let mut output = String::new();
write!(&mut output, "{}", 42)?;
writeln!(&mut output, " more")?;
```

## 格式說明符

### 位置參數

```rust
format!("{0} {1} {0}", "Hello", "World");
// "Hello World Hello"
```

### 命名參數

```rust
format!("{name} is {age} years old", name = "Alice", age = 30);
```

### 格式 trait

```rust
format!("{}", value);    // Display
format!("{:?}", value);  // Debug
format!("{:p}", value);  // Pointer
format!("{:b}", 10);     // Binary: 1010
format!("{:o}", 10);     // Octal: 12
format!("{:x}", 255);    // Hex: ff
format!("{:X}", 255);    // Hex: FF
```

### 精度

```rust
format!("{:.2}", 3.14159);    // "3.14"
format!("{:.4}", "hello");     // "hell"
format!("{:<10.5}", "word");   // "word      " (左對齊，10寬度)
```

### 填充和對齊

```rust
format!("{:>5}", "x");    // "    x" (右對齊)
format!("{:<5}", "x");    // "x    " (左對齊)
format!("{:^5}", "x");    // "  x  " (居中)
format!("{:0>5}", "x");   // "0000x" (補零)
```

## 錯誤格式化

```rust
use std::fmt;

impl fmt::Display for std::io::Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind())
    }
}
```

## Debug struct / tuple

```rust
let point = Point { x: 1, y: 2 };

// Manual
println!("{:?}", point);  // Point { x: 1, y: 2 }

// Automatic derive
#[derive(Debug)]
struct Point { x: i32, y: i32 }
```

## 本專案使用

```rust
// echo
println!("{}", args[1..].join(" "));

// tar
write!(out, "{:011o}", size)?;

// nl
println!("{:>6}{}{}", lineno, sep, line);

// dirname
println!("{}", parent);

// cmp
println!("{} {} {:o} {:o}", path1.display(), offset + j as u64, buf1[j], buf2[j]);
```

## Error

```rust
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 實現...
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 實現...
    }
}
```

## 底層機制

`println!` 展開為：
```rust
use std::io::Write;
let mut stdout = std::io::stdout();
writeln!(&mut stdout, "{}", value).unwrap();
```

## 與 C 的對比

| Rust | C |
|------|---|
| `format!` | `sprintf` |
| `println!` | `printf` |
| `write!` | `fprintf` |

## 相關模組

- `std::io`：輸出到檔案/網路
- `std::error`：錯誤格式化