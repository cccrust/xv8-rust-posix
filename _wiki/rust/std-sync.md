# std::sync

同步原語，net/ 工具使用。

## Arc

原子參考計數指標（Thread-safe）：

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);

let data_clone = Arc::clone(&data);
thread::spawn(move || {
    println!("{:?}", data_clone);
}).join().unwrap();

println!("{:?}", data);  // 主執行緒也可使用
```

## Mutex

互斥鎖：

```rust
use std::sync::Mutex;
use std::thread;

let counter = Arc::new(Mutex::new(0));

let counter_clone = Arc::clone(&counter);
thread::spawn(move || {
    let mut num = counter_clone.lock().unwrap();
    *num += 1;
}).join().unwrap();
```

### lock

```rust
let mut guard = mutex.lock().unwrap();
*guard += 1;
drop(guard);  // 明確釋放
```

## RwLock

讀寫鎖（多讀單寫）：

```rust
use std::sync::RwLock;

let lock = RwLock::new(0);

{
    let read_guard = lock.read().unwrap();
    // 多執行緒可同時讀
} // read_guard 釋放

{
    let mut write_guard = lock.write().unwrap();
    *write_guard += 1;
} // write_guard 釋放
```

## Condvar

條件變數：

```rust
use std::sync::{Arc, Mutex, Condvar};

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let (lock, cvar) = &*pair;

thread::spawn(move || {
    let (lock, cvar) = &*pair;
    *lock.lock().unwrap() = true;
    cvar.notify_one();
});

// 等待
let mut started = lock.lock().unwrap();
while !*started {
    started = cvar.wait(started).unwrap();
}
```

## Atomic

原子操作：

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static READY: AtomicBool = AtomicBool::new(false);
static COUNT: AtomicUsize = AtomicUsize::new(0);

READY.store(true, Ordering::SeqCst);
COUNT.fetch_add(1, Ordering::Relaxed);

if READY.load(Ordering::SeqCst) { }
```

### Ordering

| Ordering | 說明 |
|----------|------|
| `SeqCst` | 順序一致（最強）|
| `Acquire` | 讀取 barrier |
| `Release` | 寫入 barrier |
| `Relaxed` | 無 barrier（效能最高）|

## Once / OnceLock

單次初始化：

```rust
use std::sync::{Once, OnceLock};

static INIT: Once = Once::new();

INIT.call_once(|| {
    // 只執行一次
});

static mut VALUE: OnceLock<i32> = OnceLock::new();
VALUE.get_or_init(|| 42);
```

## mpsc

訊息傳遞：

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send(42).unwrap();
}).join().unwrap();

let value = rx.recv().unwrap();
```

### Sender / SyncSender

```rust
// 非同步（單一 producer）
let (tx, rx) = mpsc::channel();

// 同步（多 producer）
let (tx, rx) = mpsc::sync_channel(0);
```

## 本專案使用

### TCP server

```rust
use std::sync::{Arc, Mutex};

let connections = Arc::new(Mutex::new(Vec::new()));

let conns = Arc::clone(&connections);
thread::spawn(move || {
    let mut conns = conns.lock().unwrap();
    conns.push(stream);
});
```

### DNS cache

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

let cache: Arc<RwLock<HashMap<String, Vec<u8>>>> = Arc::new(RwLock::new(HashMap::new()));
```

### 計數

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0);

REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
```

## 與 xv8 的關係

xv8 目前尚未實現完整的同步原語，網路工具主要在主機上運行。

## 與 C 的對比

| Rust | POSIX | Windows |
|------|-------|---------|
| `Mutex` | `pthread_mutex` | `CRITICAL_SECTION` |
| `RwLock` | `pthread_rwlock` | `SRWLock` |
| `Condvar` | `pthread_cond` | `CONDITION_VARIABLE` |
| `Atomic` | `stdatomic.h` | `Interlocked*` |

## 死鎖避免

1. 總是以相同順序取得鎖
2. 使用 `try_lock` 而非 blocking wait
3. 減少鎖的持有時間

## 相關模組

- `std::thread`：執行緒管理
- `std::sync::atomic`：無鎖編程
- `std::collections`：HashMap 等