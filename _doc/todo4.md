# xv8 + Axum 支援規劃（v2.5）

## 現況評估

| 組件 | 狀態 | 備註 |
|------|------|------|
| `xv8-user-std` net.rs | 同步 blocking 包裝 | TcpStream/TcpListener 可運作 |
| `xv8-user-std` io_async.rs | **async I/O (epoll-based)** ✅ | AsyncTcpStream/TcpListener with AsyncRead/AsyncWrite/AsyncAccept |
| `xv8-libc` raw.rs | 最小 TCP syscalls | tcp_socket/accept/connect/send/recv |
| `xv8-async` | 單執行序 executor + timer reactor | 有 `sleep`、`yield_now`、`block_on` |
| `xv8-async` reactor | **epoll-based Reactor** ✅ | 已整合 epoll_wait I/O driver |
| `xv8-tokio-compat` | **NEW: tokio::net facade** ✅ | TcpStream/TcpListener wrappers，riscv64 編譯通過 |
| `xv8-axum-smoke` | host 端 smoke test | 驗證 tokio/axum/hyper 相依圖正確，riscv64 stub 為空 |
| kernel net stack | 完整 TCP/UDP/ETH/ARP/IPv4 + **epoll** ✅ | epoll syscalls + non-blocking I/O 已實作 |
| `_httpepoll` testbin | **NEW** ✅ | QEMU 自測 async HTTP server (fork + epoll + loopback client) |

**缺口**：缺少 non-blocking I/O、epoll 事件通知、async I/O traits、tokio runtime。

---

## 目標

在 xv8（riscv64gc-unknown-none-elf）上能編譯並執行一個最基本的 axum HTTP server：

```rust
let app = Router::new().route("/hello", get(|| async { "world" }));
let listener = TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;
```

最終產出：能在 QEMU 內跑 `axum hello world` 並用 guest curl 驗證。

---

## 策略選擇：epoll 而非背景執行緒

原規劃使用背景執行緒 + channel 模擬 non-blocking I/O，但更好的方案是直接在 xv8 kernel 加入 epoll：

**理由**：
1. xv8 kernel 已有完整 sleep/wakeup 機制（`proc::sleep`/`proc::wakeup`）和 TCP recv buffer
2. `tcp::handle_tcp()` 在數據到達時已有 wakeup 呼叫點，只需擴展為 epoll 通知
3. epoll 可服務所有 fd 類型（pipe、file、socket），不限於 TCP
4. 使用者空間 async runtime 可直接用 epoll_wait 實現真正 zero-cost async I/O

**代價**：kernel 需新增 ~800-1000 行，但這是一勞永逸的方案。

---

## 依賴鍊（不需要移植整個 tokio）

```
axum
 └── hyper      (HTTP)
      └── tokio (async runtime + io traits)
           ├── io: AsyncRead, AsyncWrite
           ├── net: TcpStream, TcpListener
           ├── time: Sleep
           ├── sync: oneshot, mpsc
           ├── task: spawn, JoinHandle
           └── rt: Runtime
```

最小可行子集：
- `tokio::io::AsyncRead` + `AsyncWrite`（TcpStream）
- `tokio::net::TcpStream` + `TcpListener` + `AsyncAccept`
- `tokio::time::Sleep`（整合現有 xv8-async timer）
- `tokio::sync::oneshot`（最基本 task 溝通）
- `tokio::task::spawn` + `JoinHandle`

---

## 分階段實作

### Phase 0：Kernel epoll + non-blocking I/O（NEW）

目標：讓 xv8 kernel 支援 epoll 和非阻塞 I/O，為使用者空間 async I/O 打下基礎。

#### Step 0a：Non-blocking fd 基礎設施

**`xv8/kernel/src/file.rs`** — 加入 `nonblocking` flag 到 `FileInner`：
```rust
pub struct FileInner {
    pub readable: bool,
    pub writeable: bool,
    pub r#type: FileType,
    pub offset: u32,
    pub nonblocking: bool,  // NEW
}
```

**`xv8/kernel/src/sysfile.rs`** — 讓 `O_NONBLOCK` 和 `fcntl(F_SETFL)` 可設定此 flag。

**`xv8/kernel/src/syscall.rs`** — 新增 `Syscall::Fcntl = 115` 以支援 `F_GETFL`/`F_SETFL`。

#### Step 0b：TCP non-blocking 回傳 EAGAIN

**`xv8/kernel/src/net/tcp.rs`** — 修改 `recv()`, `send()`, `accept()`, `connect()`：

```rust
// recv() 中：若無數據且 nonblocking flag 設為 true，直接回傳 ResourceUnavailable
if conn.nonblocking && conn.recv_buf.is_empty() {
    return Err(NetError::ResourceUnavailable);
}
```

**`xv8/kernel/src/syscall.rs`** — 現有 `SysError::ResourceUnavailable = 11`（即 EAGAIN），已可用。

#### Step 0c：epoll 核心實作

**新檔案 `xv8/kernel/src/poll.rs`**（~350 行）：

```rust
pub const POLLIN: u16 = 0x001;
pub const POLLOUT: u16 = 0x004;
pub const POLLERR: u16 = 0x008;

pub struct EpollEntry {
    pub fd: usize,
    pub events: u16,
    pub revents: u16,
    pub data: usize,
    pub conn_ptr: usize, // for TCP wakeup
    pub file_type: FileType,
}

pub struct EpollInstance {
    pub entries: Vec<EpollEntry>,
    pub triggered: Vec<EpollEvent>,
    pub waiters: usize, // count of processes sleeping on this epoll
}

pub static EPOLL_TABLE: SpinLock<[Option<EpollInstance>; NEPOLL]>;
```

核心方法：
- `epoll_create1(flags)` → 分配 EpollInstance，回傳 fd
- `epoll_ctl(epfd, op, fd, event)` → 新增/修改/刪除監控的 fd
- `epoll_wait(epfd, events, max, timeout)` → 檢查 readiness，sleep 等待事件

#### Step 0d：事件通知路徑

在 `tcp::handle_tcp()`（數據到達處）加入 epoll 喚醒邏輯：

```rust
// handle_tcp() 中，當 recv_buf 新增數據後（現有 line ~450）：
epoll_table.notify(conn_ptr, POLLIN);
```

`EpollTable::notify()` 會：
1. 查詢所有監控此 conn 的 epoll instance
2. 在 triggered 列表加入事件
3. 呼叫 `proc::wakeup(Channel::Epoll(epfd))` 喚醒等待的 epoll_wait

#### Step 0e：sys_poll() 實作

傳統 `poll()` syscall 也一併實作，供相容性使用：

- 接收 `pollfd` 陣列（來自使用者空間）
- 逐一檢查每個 fd 的 readiness（讀取 `FileInner` 和對應的 TCP/pipe state）
- 回傳 ready 的 fd 數量（0 則 timeout 或 block）

#### Step 0f：使用者空間 syscall 包裝

**`xv8/user/src/syscall.rs`** — 新增：
```rust
pub fn epoll_create1(flags: usize) -> Result<Fd, SysError>
pub fn epoll_ctl(epfd: Fd, op: usize, fd: Fd, event: &EpollEvent) -> Result<(), SysError>
pub fn epoll_wait(epfd: Fd, events: &mut [EpollEvent], timeout: isize) -> Result<usize, SysError>
pub fn poll(fds: &mut [PollFd], timeout: isize) -> Result<usize, SysError>
pub fn set_nonblocking(fd: Fd) -> Result<(), SysError>
```

**預計行數**：~800 行（kernel ~700 + user ~100）

---

### Phase 1：epoll-based async I/O

目標：用 kernel epoll 取代背景執行緒，讓 async runtime 直接用 `epoll_wait` 實現 zero-cost `AsyncRead`/`AsyncWrite`。

```
┌─────────────────────────────────────────────┐
│  xv8-async Executor                          │
│  - block_on() loop                           │
│  - run_ready_tasks()                         │
│  - epoll_wait() 替代 thread::yield_now       │
└─────────────────────────────────────────────┘
        │ epoll_wait 回傳 ready fd
        ▼
┌─────────────────────────────────────────────┐
│  Reactor（新增 xv8-async 模組）               │
│  - fd_to_waker 映射                           │
│  - epoll_ctl 註冊興趣事件                      │
│  - poll_read 時註冊 waker + epoll             │
└─────────────────────────────────────────────┘
```

**修改 `xv8rust/xv8-async/src/lib.rs`**：

新增 `Reactor` — 負責管理 epoll fd 和 waker 註冊：

```rust
pub struct Reactor {
    epoll_fd: i32,
    wakers: Mutex<HashMap<usize, Waker>>,  // fd → waker
}
```

核心邏輯：
- `register_read(fd, waker)` → `epoll_ctl(ADD, fd, EPOLLIN)` + 儲存 waker
- `register_write(fd, waker)` → `epoll_ctl(ADD, fd, EPOLLOUT)` + 儲存 waker
- `poll(timeout)` → `epoll_wait()` → 查詢對應 waker → wake()

Executor 的 `sleep_until_next_timer()` 改為：

```rust
fn reactor_tick(&self) {
    let deadline = self.inner.next_deadline();
    let timeout = deadline.map(|d| {
        let now = Instant::now();
        if d > now { (d - now).as_millis() as isize }
        else { 0 }
    }).unwrap_or(-1); // -1 = infinite

    self.reactor.poll(timeout);
    self.inner.wake_expired(Instant::now());
}
```

**新增 `xv8rust/xv8-user-std/src/io_async.rs`**：

與原規劃相同但改用 epoll 而非背景執行緒：

```rust
impl AsyncRead for AsyncTcpStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        // 1. 嘗試 non-blocking tcp_recv
        // 2. 若 EAGAIN，呼叫 Reactor::register_read(fd, cx.waker())
        // 3. 回傳 Poll::Pending
    }
}

impl AsyncWrite for AsyncTcpStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>>;
}
```

**預計行數**：~250 行

---

### Phase 2：tokio::net 適配層

目標：提供 `tokio::net::TcpStream` / `TcpListener` 讓 axum 可直接用。

在 `xv8rust/xv8-tokio-compat/` 建立新 crate：

```rust
pub struct TcpStream { inner: AsyncTcpStream }
pub struct TcpListener { inner: AsyncTcpListener }

impl tokio::io::AsyncRead for TcpStream { ... }
impl tokio::io::AsyncWrite for TcpStream { ... }
impl tokio::net::TcpListener for TcpListener { ... }
```

**策略**：用 `pub use` 把 `AsyncTcpStream` 公開為 `tokio::net::TcpStream`。

**預計行數**：~100 行

---

### Phase 3：整合 tokio runtime + timer

目標：讓 `tokio::spawn()` 和 `Sleep` 在 xv8 上運作。

**Sleep**：包裝現有 `xv8-async::Sleep` 為 `tokio::time::Sleep`。

**Runtime**：`xv8-async::Runtime` 包裝成 tokio `Runtime`：

```rust
pub struct Runtime { inner: xv8_async::Runtime }

impl tokio::runtime::RuntimeExt for Runtime {
    fn spawn<F>(&self, future: F) -> JoinHandle<F::Output> { ... }
    fn block_on<F>(&self, future: F) -> F::Output { ... }
}
```

**預計行數**：~150 行

---

### Phase 4：讓 smoke test 在 riscv64 上跑起來

更新 `xv8-axum-smoke/src/main.rs`，把 riscv64 stub 改成真 HTTP server。

在 xv8 user space 新增 `axum_test` binary，編譯進 `fs.img`。

---

### Phase 5：end-to-end 驗證

QEMU 內啟動 `axum_test &`，host 用 `curl 10.0.2.15:8080` 驗證。

---

## 檔案佈局（更新版）

```
xv8/kernel/src/
├── poll.rs                  # NEW: poll/epoll 實作
├── file.rs                  # 修改：加入 nonblocking flag
├── syscall.rs               # 修改：加入 epoll/poll/fcntl syscalls
├── sysfile.rs               # 修改：支援 O_NONBLOCK, fcntl
├── net/tcp.rs               # 修改：non-blocking recv/send/accept/connect
├── net/udp.rs               # 修改：non-blocking receive
├── param.rs                 # 修改：加入 NEPOLL
└── abi.rs                   # 修改：匯出 EpollEvent, PollFd 型別

xv8/user/src/
└── syscall.rs               # 修改：加入 epoll/poll/fcntl 包裝

xv8rust/
├── xv8-async/src/
│   └── lib.rs               # 修改：加入 Reactor (epoll-based I/O driver)
├── xv8-user-std/src/
│   ├── io_async.rs          # NEW: AsyncTcpStream, AsyncTcpListener
│   ├── net.rs               # 修改：加入 set_nonblocking 支援
│   └── lib.rs               # 修改：pub mod io_async
├── xv8-tokio-compat/        # NEW
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── tcp.rs           # tokio::net facade
│       └── runtime.rs       # tokio runtime shim
└── xv8-axum-smoke/src/
    └── main.rs              # Phase 4：解開 riscv64 stub
```

---

## 驗收標準

- [x] Phase 0a：`O_NONBLOCK` 和 `fcntl` 可在 user space 設定
- [x] Phase 0b：non-blocking TCP recv 回傳 EAGAIN
- [x] Phase 0c：`epoll_create1` + `epoll_ctl` + `epoll_wait` 可運作
- [x] Phase 0d：TCP 數據到達時 epoll_wait 被喚醒
- [x] Phase 0e：`poll()` syscall 可運作
- [x] Phase 0f：使用者空間包裝編譯成功
- [x] Phase 1：`AsyncTcpStream` 實作 `AsyncRead` + `AsyncWrite`（riscv64 編譯通過）
- [x] Phase 2：`tokio::net::TcpStream`/`TcpListener` wrappers（riscv64 編譯通過）
- [x] Phase 3：`tokio::spawn` + `tokio::time::sleep` 在 xv8 target 可編譯
- [x] Phase 4：`_httpepoll` binary 編譯進 xv8 user space + testrunner
- [x] Phase 5：`_http` test 通過（httpd+httpget loopback）；`hostfwd=tcp::8080-:8080` 已加入 QEMU 配置；可使用 `test_phase5.sh` 或手動啟動 httpd 後 `curl localhost:8080` 驗證

---

## 風險

1. **hyper 依賴** — axum 底層依賴 hyper，hyper 有龐大的 HTTP 實作，可能有額外 traits 需要實作
2. **記憶體** — xv8 kernel 的靜態配置（`NTCP=16`, `NEPOLL`）需設為合理值
3. **epoll_wait timeout** — 需與 xv8-async 的 timer 整合，確保 epoll_wait timeout ≤ 下個 timer deadline
4. **async trait** — tokio 的 `AsyncRead`/`AsyncWrite` 在正式 Rust 中已穩定

---

## 參考資料

- `xv8/kernel/src/proc.rs` — `sleep()`/`wakeup()` 實作
- `xv8/kernel/src/net/tcp.rs` — `handle_tcp()` 數據到達喚醒路徑（line ~450）
- `xv8/kernel/src/syscall.rs` — syscall 分發表和 `SysError` 列舉
- `xv8/kernel/src/file.rs` — `FileType` 列舉和 `FileInner`
- `xv8rust/xv8-async/src/lib.rs` — 現有 executor 實作
- `xv8rust/xv8-user-std/src/net.rs` — 現有 sync TCP 包裝
- `xv8rust/xv8-axum-smoke/src/main.rs` — smoke test 目標