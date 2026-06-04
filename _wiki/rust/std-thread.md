# std::thread

多執行緒支援，net/ 工具使用。

## 基本用法

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("Hello from thread");
});

handle.join().expect("Thread panicked");
```

## spawn with move

```rust
use std::thread;

let data = vec![1, 2, 3];

let handle = thread::spawn(move || {
    println!("Data: {:?}", data);
});

handle.join().unwrap();
```

## sleep

```rust
use std::thread;
use std::time::Duration;

thread::sleep(Duration::from_secs(1));
thread::sleep(Duration::from_millis(500));
thread::sleep(Duration::from_secs_f64(0.1));
```

## Thread-local storage

```rust
use std::cell::Cell;
use std::thread;

thread_local! {
    static COUNTER: Cell<i32> = Cell::new(0);
}

thread::spawn(|| {
    COUNTER.with(|c| {
        c.set(c.get() + 1);
        println!("Thread local: {}", c.get());
    });
}).join().unwrap();
```

## current

```rust
use std::thread;

let id = thread::current().id();
let name = thread::current().name();
```

## yield_now

```rust
use std::thread;

thread::yield_now();
```

## park / unpark

```rust
use std::thread;

let handle = thread::spawn(|| {
    thread::park();
    println!("Woken up!");
});

thread::sleep(Duration::from_secs(1));
handle.thread().unpark();
```

## Builder

自訂執行緒：

```rust
use std::thread;

let handle = thread::Builder::new()
    .name("worker".to_string())
    .stack_size(1024 * 1024)  // 1MB stack
    .spawn(|| {
        println!("Custom thread!");
    })
    .unwrap();

handle.join().unwrap();
```

## 優先級

```rust
use std::thread;

let handle = thread::Builder::new()
    .spawn(|| {
        // 高優先級工作
    })
    .unwrap();
```

## 本專案使用

### traceroute

```rust
// net/tools/src/bin/traceroute.rs
use std::thread;
use std::time::Duration;

let handle = thread::spawn(move || {
    for ttl in 1..max_ttl {
        // 發送 UDP 封包
        socket.send_to(&packet, &addr)?;
        thread::sleep(Duration::from_secs(1));
    }
});
```

### tcp server

```rust
// 處理每個連接一個執行緒
for stream in listener.incoming() {
    let stream = stream?;
    thread::spawn(|| {
        handle_connection(stream);
    });
}
```

### DNS 查詢

```rust
thread::spawn(move || {
    let response = resolve_dns(&query)?;
    // 處理響應
});
```

## JoinHandle

取得執行緒的結果：

```rust
let handle = thread::spawn(|| {
    42
});

let result = handle.join().unwrap();  // result = 42
```

## 記憶體模型

```rust
let data = String::from("shared");

// 錯誤：data 在主執行緒 drop 後才使用
// let handle = thread::spawn(|| {
//     println!("{}", data);  // 可能 use-after-free
// });

// 正確：使用 move
let handle = thread::spawn(move || {
    println!("{}", data);  // data moved
});
```

## 與其他語言的對比

| Rust | C++ | Go |
|------|-----|-----|
| `thread::spawn` | `std::thread` | `go func()` |
| `handle.join()` | `t.join()` | `<-ch` |
| `Arc<Mutex<T>>` | `std::mutex` | `sync.Mutex` |

## 底層機制

- **Linux/macOS**：pthreads
- **xv8**：尚未實現執行緒

## 安全性

Rust 的型別系統確保執行緒間資料傳遞的安全：
- `Send` trait：可跨執行緒傳遞
- `Sync` trait：可安全並發存取

## 相關模組

- `std::sync`：同步原語
- `std::sync::mpsc`：訊息傳遞
- `std::time`：時間相關