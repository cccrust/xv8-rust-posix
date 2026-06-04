# xv8 Rust 生態兼容路線圖

**目標：讓 xv8 能支援任意 Rust crate、完整 TCP/IP 功能、重要 Linux 功能。**

---

## ✅ v1.1–v2.3（已完成）

詳見 `_doc/todo4.md`：xv8-async runtime、QEMU async smoke test、最小 HTTP server。

---

## ✅ v2.4 — BSD sockets + xv8-user-std 基礎補缺

**已完成的 xv8-user-std 基礎補缺**（詳見 `_doc/v2.4.md`）：

- `std::io::Cursor<T>` / `BufWriter<T>` / `copy()`
- `std::net::Ipv4Addr` / `Ipv6Addr` / `IpAddr` / `SocketAddrV4` / `SocketAddrV6`
- `std::ffi::OsString`
- `std::panic::catch_unwind()` / `AssertUnwindSafe`
- `Path::is_absolute()` / `is_relative()` / `push()` / `pop()` 修復
- `std::env::temp_dir()` / `home_dir()` / `consts`
- `std::fs::canonicalize()` / `remove_dir_all()`
- sbrk allocator `dealloc` 實作（free list + coalesce）

### 待完成（v2.4 未盡項目）

### BSD sockets API shim

- 在 kernel 打造 POSIX `socket()` syscall 系列：
  - `socket(domain, type, protocol)` → 內部對應到 UDP/TCP
  - `bind(sockfd, addr, addrlen)`
  - `listen(sockfd, backlog)`
  - `accept(sockfd, addr, addrlen)`
  - `connect(sockfd, addr, addrlen)`
  - `send(sockfd, buf, len, flags)` / `recv(sockfd, buf, len, flags)`
  - `sendto(sockfd, buf, len, flags, dest, addrlen)` / `recvfrom(...)`
  - `close(sockfd)`
  - `setsockopt(sockfd, level, optname, optval, optlen)` / `getsockopt(...)`
- 保留現有自訂 TCP/UDP syscall 不衝突
- xv8-user-std `std::net` 模組直接使用 POSIX socket syscall（而非自訂 API）

### QEMU 測試

- 新增 `_netposix` QEMU 測試：使用 POSIX socket API 建立 TCP echo client
- 新增 `_cursor`、`_bufwriter` 等 xv8-user-std 單元測試
- 納入 testrunner

---

## v2.5 — select/poll/epoll + non-blocking IO

**主題：讓 async 生態系能運作**

### Kernel non-blocking mode

- `O_NONBLOCK` 旗標真實作用於所有 file I/O（read/write/accept/connect）
- Socket `SOCK_NONBLOCK` 支援
- `fcntl(fd, F_SETFL, O_NONBLOCK)` / `fcntl(fd, F_GETFL)` 實作

### select/poll/epoll

- `select(nfds, readfds, writefds, exceptfds, timeout)` — POSIX select(2)
- `poll(fds, nfds, timeout)` — POSIX poll(2)
- `epoll_create1(flags)` + `epoll_ctl(epfd, op, fd, event)` + `epoll_wait(epfd, events, maxevents, timeout)` — Linux epoll
- File / socket 層級 readiness tracking（每個 fd 的可讀/可寫狀態旗標）

### xv8-async 升級

- IO reactor 從 blocking `Poll::Ready` 改為真實 `Poll::Pending` + epoll wakeup
- `AsyncTcpListener::accept()` 在無連線時回傳 `Poll::Pending`
- `AsyncTcpStream::read()` / `write()` 在 buffer 空/滿時回傳 `Poll::Pending`
- 支援多個 concurrent connection（不再阻塞 executor）

### QEMU 測試

- 更新 `_async` 測試確認 multiple concurrent connection 可運作
- 新增 `_epoll` QEMU 測試（epoll_wait 多 fd 監控）
- 新增 `_nonblock` QEMU 測試（non-blocking connect/read）

---

## v2.6 — threading + sync 補全

**主題：讓多執行緒 crate 能編譯**

### Kernel 層

- `clone(flags, stack, parent_tid, child_tid, tls)` syscall — Linux 相容的 thread 建立
- `futex(uaddr, op, val, timeout, uaddr2, val3)` syscall — 使用者空間同步基礎
- `gettid()` syscall — thread ID
- `exit_group(status)` syscall — 終止所有 thread
- `set_tid_address(tidptr)` syscall — child tid 位址
- Per-thread 中斷控制器 + 獨立 stack
- Thread 排程（所有 thread 共享時間片）

### xv8-user-std 層

- `std::thread::spawn()` + `Builder` + `JoinHandle` + `ThreadId`
- `std::thread::LocalKey` — thread-local storage（thread_local! macro）
- `std::thread::park()` / `unpark()` — thread 阻塞/喚醒
- `std::sync::Condvar` — 基於 futex 的條件變數
- `std::sync::mpsc::channel()` / `sync_channel()` — 多 producer/single consumer channel
- `std::sync::Barrier` — 屏障同步
- `std::sync::OnceLock` — 完整 blocking once cell
- `std::sync::LazyLock` — lazy initialization
- `Mutex` / `RwLock` 從 spinlock 改為 futex-based blocking lock

### QEMU 測試

- 新增 `_thread` QEMU 測試（spawn + join + TLS + park/unpark）
- 新增 `_condvar` QEMU 測試（producer-consumer）
- 新增 `_mpsc` QEMU 測試（channel send/recv）

---

## v2.7 — fcntl + file locking + procfs

**主題：Linux 相容深化（檔案系統 + 行程資訊）**

### fcntl

- `F_DUPFD` / `F_DUPFD_CLOEXEC` — fd 複製
- `F_GETFD` / `F_SETFD` — FD_CLOEXEC
- `F_GETFL` / `F_SETFL` — O_NONBLOCK, O_APPEND 旗標
- `F_GETLK` / `F_SETLK` / `F_SETLKW` — POSIX advisory record locking
- `F_GETOWN` / `F_SETOWN` — SIGIO/SIGURG 擁有者

### File locking

- 實作 POSIX record lock table（kernel 維護 `(inode, pid, start, len, type)`）
- read lock 共用、write lock 互斥
- 行程結束自動釋放所有 lock

### procfs

- 掛載 `/proc` 虛擬檔案系統
- `/proc/self/` 基本結構：
  - `status`
  - `cmdline`
  - `mem`
  - `fd/` — 符號連結目錄
  - `maps`
- `/proc/<pid>/` — 每個行程的對應目錄
- `/proc/cpuinfo` — CPU 資訊
- `/proc/meminfo` — 記憶體資訊
- `/proc/uptime` — 系統啟動時間
- `/proc/version` — 核心版本
- `/proc/net/tcp` — TCP 連線表
- 支援 `open` / `read` / `readdir` / `fstat`

### QEMU 測試

- 新增 `_fcntl` QEMU 測試（DUPFD, GETFL/SETFL, 檔案鎖）
- 新增 `_procfs` QEMU 測試（read /proc/self/status, /proc/self/fd/）

---

## v2.8 — termios 實作 + signalfd/eventfd/timerfd + epoll 深化

**主題：完整終端機 + fd 化 event 通知**

### 完整 termios

- `tcgetattr(fd, termios)` — 真實讀取 console 屬性
- `tcsetattr(fd, TCSANOW/TCSADRAIN/TCSAFLUSH, termios)` — 真實設定 console 屬性
- 支援 flag 位元：ECHO, ICANON, ISIG, IXON, OPOST 等
- 規範模式（canonical mode）行編輯
- 原始模式（raw mode）pass-through
- VMIN / VTIME 支援
- `tcdrain()` / `tcflow()` / `tcflush()` 真實實作

### signalfd / eventfd / timerfd

- `signalfd(fd, sigmask, flags)` — 將 signal 轉為 fd 可讀
- `eventfd(initval, flags)` — 事件通知 fd
- `timerfd_create(clockid, flags)` — 計時器 fd
- `timerfd_settime(fd, flags, new, old)` — 設定 timerfd
- 三者皆可被 select/poll/epoll 監控

### QEMU 測試

- 更新 `_syscall` 測試驗證 termios flag
- 新增 `_signalfd` / `_eventfd` QEMU 測試

---

## v2.9 — Unix domain socket + DNS resolution + socket options

**主題：補全網路功能**

### Unix domain socket

- `AF_UNIX` / `AF_LOCAL` socket type
- `socket(AF_UNIX, SOCK_STREAM, 0)` — stream socket
- `socket(AF_UNIX, SOCK_DGRAM, 0)` — datagram socket
- `bind(path)` — 綁定到檔案系統路徑
- `connect(path)` — 連接到 server
- `listen()` / `accept()` — stream server
- `sendto()` / `recvfrom()` — datagram communication
- `socketpair()` — 成對 socket fd 的建立
- 抽象 socket（`\0` 前綴）支援

### DNS resolution

- `getaddrinfo(node, service, hints, res)` — POSIX 名稱解析
- `getnameinfo(sa, salen, node, serv, flags)` — 逆向解析
- `gethostbyname(name)` / `gethostbyaddr(addr, len, type)` — 傳統 API
- DNS resolver client（搭配 kernel UDP socket）
- `/etc/hosts` 支援
- `/etc/resolv.conf` 支援

### Socket options

- `SO_REUSEADDR` — 埠號重用
- `SO_KEEPALIVE` — TCP keepalive
- `SO_RCVBUF` / `SO_SNDBUF` — buffer 大小
- `SO_RCVTIMEO` / `SO_SNDTIMEO` — 超時設定
- `SO_LINGER` — 關閉行為
- `TCP_NODELAY` — 停用 Nagle
- `TCP_QUICKACK` — 立即 ACK
- `IP_TTL` — TTL 設定

### QEMU 測試

- 新增 `_unixsock` QEMU 測試（Unix socket echo）
- 新增 `_dns` 強化測試（getaddrinfo 解析 localhost）
- 新增 `_sockopt` QEMU 測試（SO_REUSEADDR, TCP_NODELAY）

---

## v2.10 — rlimit + sendfile + POSIX timers

**主題：系統資源管理 + 高效 I/O**

### rlimit

- `getrlimit(resource, rlim)` — 查詢資源限制
- `setrlimit(resource, rlim)` — 設定資源限制
- `RLIMIT_CPU`、`RLIMIT_FSIZE`、`RLIMIT_DATA`、`RLIMIT_STACK`、`RLIMIT_CORE`、`RLIMIT_RSS`、`RLIMIT_NOFILE`、`RLIMIT_AS`、`RLIMIT_NPROC`、`RLIMIT_MEMLOCK`
- 行程 fork/exec 時繼承 rlimit

### sendfile / splice

- `sendfile(out_fd, in_fd, offset, count)` — 零拷貝檔案→socket 傳輸
- `splice(fd_in, off_in, fd_out, off_out, len, flags)` — 在兩個 fd 間搬資料
- 使用 kernel 內部 buffer、不經過使用者空間

### POSIX timers

- `timer_create(clockid, sevp, timerid)` — 建立 per-process 計時器
- `timer_settime(timerid, flags, value, ovalue)` — 啟動/設定
- `timer_gettime(timerid, value)` — 查詢剩餘時間
- `timer_getoverrun(timerid)` — 溢失計數
- `timer_delete(timerid)` — 刪除計時器
- 基於 SIGALRM 或 signalfd 傳遞通知

### QEMU 測試

- 新增 `_rlimit` QEMU 測試（NOFILE, AS）
- 新增 `_sendfile` QEMU 測試（檔案→socket 傳輸）

---

## v2.11+ — IPv6 + inotify + mount + 長期工作

**主題：長期生態補全**

### IPv6

- IPv6 header / address 支援
- IPv6 UDP/TCP socket
- ICMPv6 + NDP（Neighbor Discovery Protocol）
- Dual-stack socket（AF_INET6 同時接受 IPv4）
- IPv6 DNS resolution

### inotify

- `inotify_init1(flags)` — 建立 inotify fd
- `inotify_add_watch(fd, path, mask)` — 監控檔案/目錄
- `inotify_rm_watch(fd, wd)` — 移除監控
- 事件：IN_CREATE, IN_DELETE, IN_MODIFY, IN_ACCESS, IN_ATTRIB, IN_OPEN, IN_CLOSE
- 可被 select/poll/epoll 監控

### mount / 多重檔案系統

- `mount(source, target, fstype, flags, data)` syscall
- `umount(target)` syscall
- devfs — `/dev/` 基本裝置節點自動建立
- sysfs — `/sys/` 核心資訊虛擬檔案系統
- tmpfs — RAM-backed 暫存檔案系統

### 其他

- `mremap()` — 記憶體 remapping
- `mlock()` / `mlockall()` — 記憶體鎖定
- `madvise()` — 記憶體使用建議
- `brk()` — 傳統 heap 介面（相容 glibc）
- `perf_event_open()` — 效能計數器
- `syslog()` / `klogctl()` — 核心日誌
- 完整 `O_CLOEXEC`、`O_EXCL`、`O_APPEND`、`O_DIRECTORY` 支援
- xv8-user-std 全面性補缺（`std::error::Error` impl, `std::num`, `std::borrow::Cow`, 遺漏 trait impl）

---

## 長期方向

| 項目 | 優先級 | 預估版本 |
|------|--------|---------|
| BPF / seccomp | 低 | v2.12+ |
| 容器支援（namespaces, cgroups） | 低 | v2.12+ |
| io_uring | 低 | v2.12+ |
| FUSE | 低 | v2.13+ |
| POSIX message queues | 低 | v2.13+ |
| POSIX shared memory (shm) | 低 | v2.13+ |
| POSIX 信號量 (sem) | 低 | v2.13+ |
| 完整 ptrace | 低 | v2.14+ |

---

## 實作原則

1. **每版獨立可驗證** — 每個版本有對應的 QEMU 測試（`xv8/user/testbin/`）納入 testrunner
2. **每版有 `_doc/v2.x.md`** — 版本記錄、變更、驗收結果
3. **kernel + xv8-libc + xv8-user-std 同步更新** — syscall→wrapper→std API 一氣呵成
4. **現有測試不倒退** — 所有現有 testrunner 項目持續通過
5. **優先實作介面（API）**，內部實作可逐步最佳化
