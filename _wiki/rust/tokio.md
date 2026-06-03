# tokio

非同步執行runtime，net/ 工具使用。

## 專案使用

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

## Runtime

```rust
#[tokio::main]
async fn main() {
    println!("Hello from Tokio!");
}
```

或手動建立：

```rust
use tokio::runtime::Runtime;

let rt = Runtime::new().unwrap();
rt.block_on(async {
    println!("Hello!");
});
```

## async/await

```rust
async fn fetch_data() -> String {
    // 非同步操作
    "data".to_string()
}

#[tokio::main]
async fn main() {
    let result = fetch_data().await;
    println!("{}", result);
}
```

## spawn

跨任務併發：

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        // 並發執行
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        "done"
    });

    let result = handle.await.unwrap();
    println!("{}", result);
}
```

## tokio::time

計時器：

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    sleep(Duration::from_secs(1)).await;
    println!("1 second passed");
}
```

## tokio::io

非同步 I/O：

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut file = File::open("data.txt").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    println!("{}", contents);
    Ok(())
}
```

## TCP

```rust
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            socket.write_all(&buf[..n]).await.unwrap();
        });
    }
}
```

## Channel

任務間通訊：

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });

    if let Some(v) = rx.recv().await {
        println!("Got: {}", v);
    }
}
```

## Mutex

非同步鎖：

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

let counter = Arc::new(Mutex::new(0));
let c = Arc::clone(&counter);

tokio::spawn(async move {
    let mut num = c.lock().await;
    *num += 1;
}).await.unwrap();
```

## JoinSet

並行等待多個任務：

```rust
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();

    set.spawn(async { 1 });
    set.spawn(async { 2 });
    set.spawn(async { 3 });

    while let Some(res) = set.join_next().await {
        println!("{:?}", res);
    }
}
```

## 本專案使用

### HTTP 伺服器

```rust
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

let listener = TcpListener::bind("0.0.0.0:80").await?;
loop {
    let (mut socket, _) = listener.accept().await?;
    tokio::spawn(async move {
        handle_connection(&mut socket).await;
    });
}
```

### SSH 伺服器

```rust
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    ssh_server::run().await;
});
```

## 與 std 的對比

| 特性 | tokio | std |
|------|-------|-----|
| 執行緒 | 多工作者 | 單執行緒 |
| 等待 | async await | blocking |
| I/O | 非同步 | 同步 |

## multi-thread runtime

```rust
[dependencies]
tokio = { version = "1", features = ["full"] }
```

`full` features 包含：
- `macros`：巨集支援
- `rt-multi-thread`：多執行緒 runtime
- `io-util`：非同步 I/O
- `time`：時間支援
- `net`：網路支援
- 等等

## 底層機制

tokio 使用作業系統的 async I/O（epoll/kqueue/IOCP）和工作竊取排程器。

## 錯誤處理

```rust
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let result = do_something().await?;
    Ok(())
}
```

## 相關模組

- `hyper`：HTTP 客戶端/伺服器
- `tokio::sync`：同步原語
- `tokio::io`：非同步 I/O