# std::result

Rust 的結果型別，錯誤處理的核心。

## Result 定義

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`T`：成功時的值
`E`：錯誤時的值

## 基本用法

```rust
use std::fs::File;

let file = match File::open("test.txt") {
    Ok(f) => f,
    Err(e) => {
        eprintln!("Error: {}", e);
        return;
    }
};
```

## ? 運算子

```rust
use std::fs;
use std::io;

fn read_file(path: &str) -> io::Result<String> {
    let content = fs::read_to_string(path)?;  // 自動展開 Err
    Ok(content)
}
```

## map

轉換成功值：

```rust
let result: Result<i32, &str> = Ok(5);
let doubled = result.map(|n| n * 2);  // Ok(10)
```

## map_err

轉換錯誤值：

```rust
let result: Result<i32, &str> = Err("error");
let converted = result.map_err(|e| e.to_uppercase());  // Err("ERROR")
```

## unwrap_or / unwrap_or_else

提供預設值：

```rust
let result: Result<i32, &str> = Err("error");
let value = result.unwrap_or(0);              // 0
let value = result.unwrap_or_else(|_| 0);    // 0
```

## and_then

鏈式處理：

```rust
fn get_config_path() -> Result<String, &'static str> {
    Ok("config.toml".to_string())
}

fn read_config(path: &str) -> Result<String, &'static str> {
    Ok(format!("config content of {}", path))
}

let result = get_config_path()
    .and_then(|path| read_config(&path));
```

## ok / err

轉換為 Option：

```rust
let result: Result<i32, &str> = Ok(5);
let option = result.ok();   // Some(5)

let result: Result<i32, &str> = Err("error");
let option = result.err();  // Some("error")
```

## is_ok / is_err

檢查狀態：

```rust
let result: Result<i32, &str> = Ok(5);
if result.is_ok() { }
if result.is_err() { }
```

## 本專案使用

### 檔案操作

```rust
// cat.rs
let mut file = File::open(path).unwrap_or_else(|e| {
    eprintln!("cat: {}: {}", path, e);
    std::process::exit(1);
});
```

### 網路操作

```rust
let stream = TcpStream::connect(addr).map_err(|e| {
    eprintln!("Connection failed: {}", e);
})?;
```

### 解析

```rust
let num: usize = args[i + 1].parse().map_err(|_| {
    eprintln!("Invalid number: {}", args[i + 1]);
})?;
```

## Error 類型

### std::io::Error

```rust
use std::io;

fn read_file() -> io::Result<String> {
    fs::read_to_string("test.txt")
}
```

### std::num::ParseIntError

```rust
let n: i32 = "42".parse()?;  // Result<i32, ParseIntError>
```

### 自定義錯誤

```rust
use std::fmt;

enum MyError {
    NotFound(String),
    InvalidInput,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::NotFound(s) => write!(f, "Not found: {}", s),
            MyError::InvalidInput => write!(f, "Invalid input"),
        }
    }
}
```

## 與 Option 的關係

| 類型 | 成功 | 失敗 |
|------|------|------|
| `Option<T>` | `Some(T)` | `None` |
| `Result<T, E>` | `Ok(T)` | `Err(E)` |

`Result` 是 `Option` 的推廣，區分失敗原因。

## 慣用法

```rust
// 好
let value = some_result?;

// 不好
let value = match some_result {
    Ok(v) => v,
    Err(e) => return Err(e),
};
```

## 早期返回

```rust
fn foo() -> Result<i32, Error> {
    let a = step1()?;  // 錯誤立即返回
    let b = step2()?;  // 錯誤立即返回
    Ok(a + b)
}
```

## 組合器

```rust
let result = Ok::<i32, &str>(5)
    .map(|n| n + 1)           // Ok(6)
    .map_err(|e| e.to_string()) // Ok(6) (不變)
    .and_then(|n| Ok(n * 2)); // Ok(12)
```

## 相關模組

- `std::io`：I/O 錯誤
- `std::fmt`：錯誤格式化
- `std::option`：Option 枚舉