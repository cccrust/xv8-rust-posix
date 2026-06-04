# std::option

Rust 的可選型別。

## Option 定義

```rust
pub enum Option<T> {
    Some(T),
    None,
}
```

`Some(T)`：有值
`None`：無值

## 基本用法

```rust
let value: Option<i32> = Some(5);

match value {
    Some(n) => println!("Got: {}", n),
    None => println!("No value"),
}
```

## unwrap_or

取得值或預設：

```rust
let value: Option<i32> = None;
let n = value.unwrap_or(0);  // 0
```

## ? 運算子

```rust
fn get_first(items: &[i32]) -> Option<i32> {
    items.first().copied()
}

fn get_first_squared(items: &[i32]) -> Option<i32> {
    let first = items.first().copied()?;  // None 的話直接返回
    Some(first * first)
}
```

## map

轉換值：

```rust
let value: Option<i32> = Some(5);
let doubled = value.map(|n| n * 2);  // Some(10)

let value: Option<i32> = None;
let doubled = value.map(|n| n * 2);  // None
```

## and_then

鏈式處理：

```rust
let value: Option<i32> = Some(5);
let result = value.and_then(|n| {
    if n > 0 { Some(n * 2) } else { None }
});  // Some(10)
```

## or / or_else

提供替代值：

```rust
let value: Option<i32> = None;
let n = value.or(Some(10));         // Some(10)
let n = value.or_else(|| Some(10)); // Some(10)
```

## filter

保留滿足條件的值：

```rust
let value: Option<i32> = Some(5);
let result = value.filter(|n| n > &3);  // Some(5)
let result = value.filter(|n| n > &10); // None
```

## 與 Result 的轉換

```rust
let option: Option<i32> = Some(5);
let result: Result<i32, ()> = option.ok_or(());
// Ok(5)

let option: Option<i32> = None;
let result: Result<i32, ()> = option.ok_or(());
// Err(())
```

## is_some / is_none

```rust
let value: Option<i32> = Some(5);
if value.is_some() { }
if value.is_none() { }
```

## 本專案使用

### 路徑操作

```rust
let path = Path::new("/tmp/file");
if let Some(parent) = path.parent() {
    println!("Parent: {:?}", parent);
}
```

### 環境變數

```rust
let path = std::env::var_os("PATH");
if let Some(p) = path {
    // ...
}
```

### 字串解析

```rust
let num: Option<usize> = args[i + 1].parse().ok();
if let Some(n) = num {
    // ...
}
```

### 字元搜尋

```rust
let s = "hello";
if let Some(pos) = s.find('o') {
    println!("Found at {}", pos);
}
```

## 慣用法

### if let

```rust
if let Some(value) = option {
    println!("{}", value);
}
```

### while let

```rust
while let Some(item) = iterator.next() {
    println!("{}", item);
}
```

## 與指標的對應

| Option | Raw Pointer |
|--------|-------------|
| `Some(ptr)` | `NonNull::new(ptr)` |
| `None` | `null` |

## 與其他語言的對比

| Rust | Java | C++ |
|------|------|-----|
| `Option<T>` | `Optional<T>` | `std::optional<T>` |
| `Some(T)` | `Optional.of(T)` | `std::make_optional(T)` |
| `None` | `Optional.empty()` | `(not exists)` |

## Null 安全

Rust 的 `Option` 強制處理 null 情况，編譯期杜絕 null 指標錯誤。

## 相關模組

- `std::result`：Result 枚舉
- `std::iter`：Iterator methods for Option