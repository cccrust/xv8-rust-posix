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

## 🚧 v3.1 — Process + Wait Correctness

**Reference:** `library/std/src/process.rs` (wait, wait_with_output, try_wait)
**Reference:** `library/std/src/sys/process/unix/unix.rs` (Process::wait via waitpid)
**Reference:** `library/std/src/sys/process/unix/common.rs` (read_output via poll)

- [x] `process::Child::wait()`: loop `xv8_libc::wait()` until PID matches (emulate `waitpid`)
- [x] `process::Child::try_wait()`: non-blocking check (WNOHANG equivalent)
- [x] `process::Child::wait_with_output()`: read stdout/stderr **before** wait using `poll()` (avoid deadlock)
- [x] Cache `ExitStatus` in `Child` after successful wait (idempotent calls)
- [x] `process::Command::spawn()` error handling: close pipes on `fork` failure

## 🚧 v3.2 — Futex-based Park + Condvar

**Reference:** `library/std/src/sys/sync/thread_parking/futex.rs` (Parker state machine)
**Reference:** `library/std/src/sys/pal/unix/futex.rs` (futex_wait/futex_wake/futex_wake_all wrappers)
**Reference:** `library/std/src/sys/pal/unix/sync` (Condvar via futex)

- [ ] Add `futex_wait`/`futex_wake` syscall wrappers to `xv8-libc` (kernel already has sleep/wakeup)
- [ ] Rewrite `thread::park()`/`unpark()` to use futex instead of pipe (no per-thread FD)
- [ ] Rewrite `sync::Condvar` to use futex (supports multiple waiters)
- [ ] Remove per-thread pipe FDs from `Tcb` (simplify thread spawn)

## 🚧 v3.3 — thread::Builder + Environment Safety

**Reference:** `library/std/src/thread/builder.rs` (Builder with name, stack_size)
**Reference:** `library/std/src/thread/lifecycle.rs` (spawn_unchecked with error handling)
**Reference:** `library/std/src/env.rs` (set_var/remove_var marked unsafe)

- [ ] `thread::Builder` struct: `new()`, `.name()`, `.stack_size()`, `.spawn()`
- [ ] `thread::spawn()` as shortcut using `Builder::default()`
- [ ] `Thread::name()` accessor
- [ ] `env::set_var`/`remove_var` marked `unsafe` (match Rust std)
- [ ] `thread::spawn` error handling: `sbrk` failure, `clone_tls` failure → return `io::Error`

## 🚧 v3.4 — File + IO Completion

**Reference:** `library/std/src/sys/pal/unix/fs.rs` (seek SEEK_END, append, metadata)

- [ ] `fs::File::seek(SeekFrom::End)`: use kernel file size
- [ ] `fs::OpenOptions::append(true)`: `O_APPEND` flag
- [ ] `fs::canonicalize`: resolve `.`/`..` in path
- [ ] `io::copy()`: efficient `read_to_end`→`write_all` loop
- [ ] `io::empty()`/`io::repeat()`/`io::sink()`: no-op readers/writers

## 📋 Future

### v3.5 — Performance
- Mutex: futex-based (no spin-then-pipe)
- Alloc: sbrk chunk pre-allocation
- Instant: `clock_gettime` fallback

### v3.6 — Advanced API
- `net::lookup_host`
- `os::unix::process::CommandExt`
- `os::unix::net` (UnixStream)
- `panic::set_hook`
- `Barrier`, `OnceCell`, `LazyLock`

### v3.7 — Async I/O
- TcpStream/TcpListener integration with xv8-async epoll reactor
- `AsyncRead`/`AsyncWrite` impls

### v3.8 — Testing
- Host-side unit tests (`#[cfg(test)]`)
- QEMU integration: `_thread_v3` test binary using xv8-user-std::thread API
- Real-world crate compat: serde_json, regex, httparse

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
| `process::{Command,Child}` | `process.rs` | ⚠️ v3.1 fixes |
| `fs::{File,OpenOptions,ReadDir}` | `fs.rs` | ⚠️ missing append |
| `net::{Tcp,TcpListener,Udp}` | `net.rs` | ✅ |
| `sync::{Mutex,Condvar,RwLock}` | `sync.rs` | ⚠️ Condvar 1-waiter |
| `time::{Duration,Instant,SystemTime}` | `time.rs` | ✅ |
| `sys/pal/{futex,thread_parking}` | not yet | ❌ v3.2 |
