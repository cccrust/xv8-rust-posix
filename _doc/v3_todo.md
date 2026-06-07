# xv8-rust-posix Development Plan

Based on [Porting Rust standard library (OSDev Wiki)](https://wiki.osdev.org/Porting_Rust_standard_library) + [`/Users/Shared/ccc/github/rust/library/std/`](https://github.com/rust-lang/rust) reference.

Target: `riscv64gc-unknown-none-elf` + `xv8-user-std` crate as std overlay.

---

## ✅ v3.0 (Completed)

- Kernel: `clone_tls`, CLONE_VM/CLONE_SIGHAND/CLONE_SETTLS
- `xv8-user-std::thread`: spawn, join, park, unpark, sleep, yield_now
- Core PAL: io, fs, path, env, process, net, sync, mpsc, time, ffi, runtime
- Test: 8 QEMU core tests pass

---

## ✅ v3.1 (Completed)

- `process::Child::wait()`: loop `xv8_libc::wait()` until PID matches (emulate `waitpid`)
- `process::Child::try_wait()`: non-blocking check (WNOHANG equivalent)
- `process::Child::wait_with_output()`: read stdout/stderr **before** wait using `poll()` (avoid deadlock)
- Cache `ExitStatus` in `Child` after successful wait (idempotent calls)
- `process::Command::spawn()` error handling: close pipes on `fork` failure

## ✅ v3.2 (Completed)

- Kernel: `futex` syscall (FUTEX_WAIT/FUTEX_WAKE), `Channel::Address`, `wakeup_n`
- `xv8-libc`: `futex()` wrapper, `FUTEX_WAIT`/`FUTEX_WAKE` constants
- `thread::park()`/`unpark()`: pipe → futex 3-state protocol (EMPTY/NOTIFIED/PARKED)
- `sync::Condvar`: pipe → futex counter (supports multiple waiters)
- Removed per-thread pipe FDs from `Tcb`

## ✅ v3.3 (Completed)

- `thread::Builder` struct: `new()`, `.name()`, `.stack_size()`, `.spawn()` → `io::Result`
- `thread::spawn()` as shortcut using `Builder::default()`
- `Thread::name()` accessor (via TCB leaked name ptr)
- `env::set_var`/`remove_var` marked `unsafe` (match Rust std)
- `thread::spawn` error handling: `sbrk`/`clone_tls` failure → return `io::Error`

## ✅ v3.4 (Completed)

- `io::copy()`: 4096-byte buffer loop (`read` → `write_all`)
- `io::empty()`/`io::repeat(u8)`/`io::sink()`: noop readers/writer
- `fs::canonicalize()`: resolve `.`/`..` using `env::var("PWD")` for relative paths
- Confirmed: `SeekFrom::End` (lseek) + `OpenOptions::append` already work

## ✅ v3.5 (Completed)

- Mutex: futex-based 3-state (UNLOCKED/LOCKED/CONTENDED) — no pipe FDs
- Alloc: sbrk chunk pre-allocation (64KB bump allocator)
- Instant: still uses `uptime()` (no `clock_gettime` syscall in xv8)

## ✅ v3.6 (Completed)

- `net::lookup_host()` — UDP DNS query to 8.8.8.8:53 (type A record)
- `os::unix::process::CommandExt` — uid/gid trait
- `os::unix::net::UnixStream` — pipe-based pair()
- `panic::set_hook`/`take_hook` — runtime panic hook
- `sync::Barrier` — count-down + condvar notify_all

## ✅ v3.7 (Completed)

- `xv8-async`: epoll reactor + AsyncTcpStream/AsyncTcpListener (poll-based I/O)
- `xv8-tokio-compat`: TcpStream/TcpListener wrapping async types
- `AsyncRead`/`AsyncWrite` impls for TcpStream with AsyncReadExt/AsyncWriteExt

## ✅ v3.8 (Completed)

- QEMU integration: `_thread_v3` test binary (named threads, stack size, multi-named)
- 18/18 QEMU tests pass (9 core + 6 net + 3 async)

---

## 🧱 Architecture: Rust std vs xv8

| Rust std component | xv8 equivalent | Status |
|---|---|---|
| `target triple` | `riscv64gc-unknown-none-elf` | ✅ |
| `sys/pal/unix/` | `xv8-user-std/src/` (PAL) | ✅ |
| Global allocator | `SbrkAlloc` (sbrk-based) | ✅ |
| TLS (`#[thread_local]`) | `CLONE_SETTLS` + `tp` reg + `Tcb` | ✅ |
| `_start` / `lang_start` | `runtime.rs` (standalone feature) | ✅ |
| `io::{stdio, print}` | `io.rs` via `xv8_libc::write` | ✅ |
| CLI args | `env::args()` via xv8 `argv` | ✅ |
| Env vars | `env::{var,set_var,remove_var}` | ⚠️ unsafety |
| `thread::{Builder,spawn, park}` | `thread.rs` | ✅ v3.0 |
| `process::{Command,Child}` | `process.rs` | ✅ v3.1 |
| `fs::{File,OpenOptions,ReadDir}` | `fs.rs` | ⚠️ missing append |
| `net::{Tcp,TcpListener,Udp}` | `net.rs` | ✅ |
| `sync::{Mutex,Condvar,RwLock}` | `sync.rs` | ✅ Condvar v3.2 |
| `time::{Duration,Instant,SystemTime}` | `time.rs` | ✅ |
| `sys/pal/{futex,thread_parking}` | `thread.rs` + kernel | ✅ v3.2 |

---

## v4.x — Linux 獨有功能

> 詳細計畫請見 [`v4_todo.md`](v4_todo.md)

| 版本 | 主題 | 新增 syscalls |
|------|------|---------------|
| v4.1 | Notification FD | eventfd, signalfd, timerfd |
| v4.2 | 新 FD 類型 | memfd_create, pidfd_open |
| v4.3 | 零拷貝 I/O | splice, tee, vmsplice |
| v4.4 | 通用工具 | getrandom, close_range, prctl |
| v4.5 | 檔案監控 | inotify_init1, add_watch, rm_watch |
| v4.6 | os::linux 模組完整化 | — |
| v4.7 | 整合測試 + QEMU 驗證 | — |

## v5.x — Docker / 容器基礎設施

> 詳細計畫請見 `v4_todo.md#v5x--docker--容器基礎設施`

| 版本 | 主題 | 核心新增 |
|------|------|---------|
| v5.1 | Namespaces | clone(CLONE_NEW*), unshare, setns |
| v5.2 | cgroups v2 | cgroupfs + CPU/Mem/PID controllers |
| v5.3 | 安全隔離 | Capabilities + seccomp-BPF |
| v5.4 | OverlayFS | 堆疊式聯合掛載檔案系統 |
| v5.5 | 容器網路 | veth pair, bridge, NAT |
| v5.6 | 容器執行環境 | pivot_root, sethostname, container lifecycle |
| v5.7 | xv8-container 工具 | Docker-like CLI |
