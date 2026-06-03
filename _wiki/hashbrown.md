# hashbrown

高效能 HashMap 實現，xv8-user-std 使用。

## 專案使用

```toml
[dependencies]
hashbrown = "0.15"
```

## 主要 API

與標準庫的 `std::collections::HashMap` API 相同：

```rust
use hashbrown::HashMap;

let mut map: HashMap<&str, i32> = HashMap::new();
map.insert("apple", 1);
map.insert("banana", 2);

if let Some(value) = map.get("apple") {
    println!("{}", value);
}
```

## 與 std::collections::HashMap 的差異

| 特性 | hashbrown | std |
|------|-----------|-----|
| 依賴 | 無 | 有 |
| no_std | 是 | 否 |
| 效能 | 優化 | 標準 |
| API | 相容 | 標準 |

## 效能優化

### 減少指標 chasing

hashbrown 使用線性探測和 Robin Hood 雜湊，減少記憶體訪問。

### SIMD 加速

某些實現使用 SIMD 指令加速雜湊計算。

### no_std 支援

可在 `no_std` 環境中使用：

```rust
#![no_std]
extern crate hashbrown;
```

## Raw API

低層級控制：

```rust
use hashbrown::raw::{RawTable, Equivalent};

let mut table = RawTable::new();
table.insert(0, "zero");
table.insert(1, "one");
```

## HashMap 迭代

```rust
for (key, value) in &map {
    println!("{}: {}", key, value);
}

for key in map.keys() { }
for value in map.values() { }
```

## 本專案使用

xv8-user-std 需要在 `no_std` 環境中提供 HashMap 功能：

```toml
[package]
name = "xv8-user-std"

[dependencies]
hashbrown = "0.15"
```

## Default

```rust
let map = HashMap::default();
// 等同於 HashMap::new()
```

## 容量

```rust
let mut map = HashMap::with_capacity(100);
map.reserve(50);  // 確保有足夠空間
let capacity = map.capacity();
```

## 清除

```rust
let mut map = HashMap::new();
map.insert("a", 1);
map.clear();  // 所有鍵值對移除
```

## 移除

```rust
let mut map = HashMap::new();
map.insert("a", 1);

if let Some(value) = map.remove("a") {
    println!("Removed: {}", value);
}
```

## 取得或插入

```rust
let value = map.entry("key").or_insert_with(|| 42);
```

## 使用場景

### std 的替代

在效能敏感或記憶體受限的環境使用。

### no_std 環境

標準庫 HashMap 需要 `std`，hashbrown 可在嵌入式使用。

### 遊戲引擎

遊戲通常需要快速的鍵值查詢，hashbrown 提供優勢。

## 設計

hashbrown 受到 SwissTable 設計的啟發，使用：
- 雜湊表的 SIMD 向量化
- 線性探測減少衝突
- Robin Hood 雜湊改善最壞情況

## 與 Rust 標準庫的關係

Rust 1.56+ 將 hashbrown 的實現用於 `std::collections::HashMap`。

```rust
// 實際上 std::collections::HashMap 內部使用 hashbrown
use std::collections::HashMap;
let map = HashMap::new();  // 內部是 hashbrown::HashMap
```

## 未來發展

hashbrown 持續優化，可能包含：
- 更快的雜湊函數
- 更好的記憶體局部性
- 更多的 SIMD 優化

## 相關模組

- `std::collections`：標準集合
- `ahash`：更快的雜湊函數
- `rustc-hash`：編譯器使用的雜湊