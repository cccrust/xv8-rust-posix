# std::collections

Rust 的集合資料結構，本專案大量使用 HashMap。

## HashMap

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key", "value");
map.insert("name", "Alice");

if let Some(val) = map.get("key") {
    println!("{}", val);
}

for (k, v) in &map {
    println!("{}: {}", k, v);
}
```

## 基本操作

```rust
let mut scores = HashMap::new();

scores.insert("Alice", 10);
scores.insert("Bob", 15);

// 取得（回傳 Option）
let alice_score = scores.get("Alice");  // Some(&10)

// 更新
scores.insert("Alice", 20);  // 覆蓋

// 刪除
scores.remove("Bob");

// 檢查存在
if scores.contains_key("Alice") { }
```

## entry API

```rust
use std::collections::hash_map::Entry;

let mut count: HashMap<String, i32> = HashMap::new();

// 插入或更新
count.entry("word".to_string()).or_insert(0);
*count.entry("word".to_string()).or_insert(0) += 1;

// 處理已存在的值
match count.entry(key) {
    Entry::Vacant(e) => { e.insert(1); }
    Entry::Occupied(e) => { *e.into_mut() += 1; }
}
```

## VecDeque

雙端佇列，用於佇列和堆疊：

```rust
use std::collections::VecDeque;

let mut deque = VecDeque::new();
deque.push_back(1);
deque.push_front(0);
let front = deque.pop_front();  // Some(0)
```

## HashSet

集合（不重複）：

```rust
use std::collections::HashSet;

let mut set = HashSet::new();
set.insert("apple");
set.insert("banana");

if set.contains("apple") { }
for item in &set { }
```

## BTreeMap / BTreeSet

有序的樹結構：

```rust
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("a", 1);
map.insert("b", 2);
for (k, v) in &map {
    println!("{}: {}", k, v);  // 按 key 排序輸出
}
```

## 本專案使用

### shell 的環境變數

```rust
// sh.rs
use std::collections::HashMap;

let mut variables: HashMap<String, String> = HashMap::new();
variables.insert("PATH".to_string(), "/bin:/usr/bin".to_string());
```

### find 的路徑追蹤

```rust
let mut visited: HashSet<String> = HashSet::new();
if !visited.contains(&path_str) {
    visited.insert(path_str);
    // 處理目錄
}
```

### grep 的比對

```rust
let pattern = Regex::new(&args[1])?;
let mut matches: Vec<String> = Vec::new();
```

## 效能考量

- `HashMap`：O(1) 查詢，但無序
- `BTreeMap`：O(log n) 查詢，有序
- `Vec`：O(1) 索引

## 記憶體

HashMap 需要額外記憶體儲存 bucket 和指標。

## Default

```rust
let mut map: HashMap<String, Vec<i32>> = HashMap::new();
map.entry("key").or_insert_with(Vec::new);
```

## 與 Rust 內建陣列的比較

```rust
let arr = ["a", "b", "c"];          // 編譯期大小固定
let vec = vec!["a", "b", "c"];      // 動態大小
let map = HashMap::from([            // Rust 1.56+
    ("a", 1),
    ("b", 2),
]);
```

## 迭代順序

- `HashMap`/`HashSet`：不保證順序
- `BTreeMap`/`BTreeSet`：按 key 排序
- `Vec`/`VecDeque`：插入順序

## 相關模組

- `std::sync`：並發集合
- `alloc`：堆積配置