# xv8-std Plan：自訂 sysroot 讓 xv8 支援 std（精簡版）

## 目標

讓 `posix/tools/` **不修改任何一行**就能在 xv8 上編譯執行。用 stub 填補目前不支援的功能，只實作用到的 `std` 子集。

---

## 範圍：posix/tools/ 實際用到的 std 模組

盤點完 100+ 個 binary 後，實際用到的 `std` 子集極小：

| std 模組 | 用到的 binary 數 | xv8 syscall 對應 |
|---|---|---|
| `std::io::BufRead` | ~20 | `read` |
| `std::fs` | ~15 | `open`/`read`/`write`/`stat`/`chdir` |
| `std::path::Path` | ~15 | 無（純指標處理） |
| `std::env::args` | ~12 | `exec` 傳參 |
| `std::process::exit` | ~10 | `exit` |
| `std::io::{Read, Write}` | ~8 | `read`/`write` |
| `std::fs::File` | ~6 | `open`/`read`/`write`/`lseek` |
| `std::env` | ~5 | `getenv`/`chdir` |
| `std::process::{Command, Stdio}` | 3（sh, xargs, nohup） | `fork`/`exec`/`wait` |
| `std::time::{Duration, UNIX_EPOCH}` | 2（sleep, touch） | 目前無精確時鐘 |
| `std::os::unix::fs::{MetadataExt, PermissionsExt}` | 2（ls, mkdir） | `fstat`/`chmod` |
| `std::ffi::{CStr, CString}` | 2（touch, ls） | 無（指標處理） |
| `std::collections::HashMap` | 4（sort, join, m4, tsort） | 無（純記憶體） |
| `std::thread` | 1（sleep） | 目前無執行緒 |
| `std::time::SystemTime` | 1（touch） | 目前無時鐘 |

**不需要實作的（完全沒用到）：**
- `std::net`、`std::sync`（Mutex、Condvar 等）、`std::os::unix::process`、`std::env::home_dir`、`std::ffi::OsString`、`std::panic`

---

## 整體架構

```
xv8-std/
├── xv8-libc/          # syscall wrapper（C-like API）
│   └── src/lib.rs
└── xv8-std-overlay/   # 只實作 posix/tools/ 用到的部分
    └── src/
        ├── lib.rs
        ├── io.rs          # Read, Write, BufRead, Seek
        ├── fs.rs          # File, OpenOptions, read_dir, Metadata
        ├── path.rs        # Path, PathBuf（純指標包裝）
        ├── env.rs         # args(), vars()
        ├── process.rs     # Command, Stdio, exit
        ├── time.rs        # Duration, UNIX_EPOCH（stub: SystemTime）
        ├── thread.rs      # thread::sleep（stub）
        ├── ffi.rs         # CStr, CString（指標包裝）
        ├── collections.rs # HashMap（基於 core 的 no_std HashMap）
        └── os/
            └── unix/
                └── fs.rs  # MetadataExt, PermissionsExt
```

---

## Phase 0：建立 target specification

在 `xv8/riscv64gc-unknown-xv8-elf.json`（或直接用現有的 `riscv64gc-unknown-none-elf`，在 `.cargo/config.toml` 裡 `os = "xv8"`）。

```toml
# xv8/.cargo/config.toml
[build]
target = "riscv64gc-unknown-none-elf"

[target.riscv64gc-unknown-none-elf]
# 現有 QEMU runner 設定保留
# 新增：sysroot 指向 xv8-std overlay
```

---

## Phase 1：實作 `xv8-libc`（最少必要 syscall 包裝）

`xv8-libc` 的目標是提供 C-like 介面，讓 `xv8-std-overlay` 可以呼叫。

實作 `posix/tools/` 需要的符號（其餘 stub）：

```rust
// xv8-libc/src/lib.rs
#![no_std]

pub mod io {
    use crate::syscall::*;

    pub fn read(fd: usize, buf: &mut [u8]) -> isize {
        sys_read(fd, buf.as_mut_ptr(), buf.len())
    }

    pub fn write(fd: usize, buf: &[u8]) -> isize {
        sys_write(fd, buf.as_ptr(), buf.len())
    }
}

pub mod fs {
    pub fn open(path: &CStr, flags: u32, mode: u32) -> isize {
        sys_open(path.as_ptr(), flags, mode)
    }
    pub fn stat(path: &CStr, stat_buf: *mut Stat) -> isize { sys_stat(path.as_ptr(), stat_buf) }
    pub fn fstat(fd: usize, stat_buf: *mut Stat) -> isize { sys_fstat(fd, stat_buf) }
    pub fn chdir(path: &CStr) -> isize { sys_chdir(path.as_ptr()) }
    pub fn readdir(fd: usize) -> isize { sys_readdir(fd) }
}

pub mod proc {
    pub fn exit(code: i32) -> ! { sys_exit(code) }
    pub fn fork() -> isize { sys_fork() }
    pub fn wait(status: *mut i32) -> isize { sys_wait(status) }
    pub fn exec(path: &CStr, argv: *const *const u8) -> isize { sys_exec(path, argv) }
    pub fn getpid() -> usize { sys_getpid() }
    pub fn kill(pid: usize, sig: i32) -> isize { sys_kill(pid, sig) }
}

pub mod mem {
    pub fn sbrkinc(n: isize) -> *mut u8 { sys_sbrk(n) }
}

pub mod net {
    // stub -- posix/tools/ 沒有用到網路功能
    pub fn socket(...) -> isize { -1 }
}
```

**不需要實作的**（posix/tools/ 沒用到）：`clone`、`mmap`、`munmap`、`mprotect`、`pipe`、`dup`、`ioctl`、`time`（時鐘）、`socket`（網路）、`prlimit64`、`getrusage`

---

## Phase 2：實作 `xv8-std-overlay`（核心：std trait 實作）

只實作用到的部分。其餘 panics 或 returns stub error。

### 2.1 `std::io` — 實作 Read, Write, BufRead, Seek

```rust
// xv8-std-overlay/src/io.rs

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = xv8_libc::io::read(0, buf);
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(n as usize) }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = xv8_libc::io::read(self.fd, buf);
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(n as usize) }
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = xv8_libc::io::write(1, buf);
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(n as usize) }
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl BufRead for Stdin {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // 用內部緩衝區實作
    }
    fn consume(&mut self, amt: usize) { ... }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let off = xv8_libc::fs::lseek(self.fd, pos as i64);
        if off < 0 { Err(io::Error::last_os_error()) } else { Ok(off as u64) }
    }
}
```

### 2.2 `std::fs` — 實作 File, Metadata, read_dir

`posix/tools/` 用到的 `std::fs` API：
- `fs::read_to_string(path)` → `sys_open` + `read`
- `fs::write(path, data)` → `sys_open(write) + write`
- `fs::metadata(path)` → `sys_stat`
- `fs::symlink_metadata(path)` → `sys_stat`（xv8 沒區分）
- `fs::read_dir(path)` → `sys_open + sys_readdir`
- `File::open(path)` → `sys_open`
- `File::read/read_to_end` → `sys_read`
- `OpenOptions::new().read(true).write(true).open()`

```rust
// xv8-std-overlay/src/fs.rs

pub struct File { fd: usize }
pub struct OpenOptions { ... }
pub struct ReadDir { ... }
pub struct Metadata { stat: Stat }

impl File {
    pub fn open(path: &Path) -> io::Result<Self> {
        let fd = xv8_libc::fs::open(path.to_c_str(), O_RDONLY, 0);
        if fd < 0 { Err(io::Error::last_os_error()) } else { Ok(File { fd: fd as usize }) }
    }
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { ... }
}

impl Read for File { ... }
impl Write for File { ... }
impl Seek for File { ... }

pub fn read_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

pub fn read_dir(path: &Path) -> io::Result<ReadDir> { ... }

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> { ... }
}
```

### 2.3 `std::path` — Path, PathBuf

這幾乎是純指標/切片操作，不需要 syscall，直接包裝：

```rust
// xv8-std-overlay/src/path.rs
pub struct Path { inner: [u8] }
pub struct PathBuf { inner: Vec<u8> }  // Vec 在 no_std 用 alloc crate

impl Path {
    pub fn new(s: &[u8]) -> &Path { Path { inner: *s } }
    pub fn to_str(&self) -> Option<&str> { ... }
    pub fn file_name(&self) -> Option<&Path> { ... }
    pub fn exists(&self) -> bool { xv8_libc::fs::stat(self.to_c_str(), ...).is_ok() }
    pub fn is_dir(&self) -> bool { self.metadata().map(|m| m.is_dir()).unwrap_or(false) }
}
```

### 2.4 `std::env` — args(), vars(), current_dir()

```rust
// xv8-std-overlay/src/env.rs
pub fn args() -> Args { Args }
pub struct Args { ... }
impl Iterator for Args { type Item = String; fn next(&mut self) -> Option<String> { ... } }

pub fn vars() -> Vars { Vars }  // stub: empty for now
pub fn current_dir() -> io::Result<PathBuf> {
    let mut buf = [0u8; 256];
    let n = xv8_libc::fs::getcwd(&mut buf);
    if n < 0 { Err(io::Error::last_os_error()) } else { Ok(PathBuf::from(&buf[..n as usize])) }
}
```

### 2.5 `std::process` — Command, Stdio, exit

```rust
// xv8-std-overlay/src/process.rs
pub fn exit(code: i32) -> ! { xv8_libc::proc::exit(code); loop {} }

pub struct Command { prog: String, args: Vec<String>, ... }
impl Command {
    pub fn new(prog: &str) -> Self { Command { prog: prog.into(), .. } }
    pub fn arg(&mut self, arg: &str) -> &mut Self { self.args.push(arg.into()); self }
    pub fn spawn(&mut self) -> io::Result<Child> {
        let pid = xv8_libc::proc::fork();
        if pid == 0 {
            // 在子行程：exec
            xv8_libc::proc::exec(...);
            xv8_libc::proc::exit(1); // 如果 exec 失敗
        }
        Ok(Child { pid })
    }
}

pub enum Stdio { Inherit, Null, Piped, ... }
```

### 2.6 `std::thread` — thread::sleep（stub）

```rust
// xv8-std-overlay/src/thread.rs
pub fn sleep(_dur: Duration) {
    // xv8 沒有精確時鐘。busy wait 作為 stub。
    // 之後有了定時器 interrupt 再實作真正的 sleep。
}
```

### 2.7 `std::time` — Duration, UNIX_EPOCH

```rust
// xv8-std-overlay/src/time.rs
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Duration { pub secs: u64, pub nanos: u32 }

impl Duration {
    pub fn new(secs: u64, nanos: u32) -> Self { ... }
    pub fn as_secs(&self) -> u64 { self.secs }
    pub fn as_millis(&self) -> u64 { self.secs * 1000 + (self.nanos / 1_000_000) as u64 }
}

pub const UNIX_EPOCH: SystemTime = SystemTime { secs: 0 };

pub struct SystemTime { secs: u64 }
impl SystemTime {
    pub fn now() -> Self { Self { secs: 0 } } // stub: 沒有時鐘
    pub fn duration_since(&self, t: SystemTime) -> Result<Duration, ()> { ... }
    pub fn checked_add(&self, d: Duration) -> Option<SystemTime> { ... }
}
```

### 2.8 `std::ffi` — CStr, CString

```rust
// xv8-std-overlay/src/ffi.rs
pub struct CStr { inner: [u8] }
impl CStr {
    pub fn from_ptr(p: *const c_char) -> &CStr { ... }
    pub fn to_bytes(&self) -> &[u8] { ... }
    pub fn to_string_lossy(&self) -> String { ... }
}

pub struct CString { inner: Vec<u8> }
impl CString {
    pub fn new(s: &str) -> Result<Self, NulError> { ... }
    pub fn as_c_str(&self) -> &CStr { ... }
}
```

### 2.9 `std::collections` — HashMap（基於 `core` + `alloc`）

`std::collections::HashMap` 在 `std` 外部只是 re-export `alloc::collections::HashMap`。在 `no_std` 環境下直接使用 `alloc` crate 提供的實作即可，不需要另外包裝。

```rust
extern crate alloc;
pub use alloc::collections::HashMap;
```

### 2.10 `std::os::unix::fs` — MetadataExt, PermissionsExt

```rust
// xv8-std-overlay/src/os/unix/fs.rs

pub trait MetadataExt {
    fn mode(&self) -> u32;
    fn uid(&self) -> u32;
    fn gid(&self) -> u32;
    fn size(&self) -> u64;
    fn mtime(&self) -> i64;
    fn nlink(&self) -> u32;
    fn blocks(&self) -> u64 { 0 } // stub
}

pub trait PermissionsExt {
    fn mode(&self) -> u32;
    fn set_mode(&mut self, mode: u32);
}

impl MetadataExt for fs::Metadata {
    fn mode(&self) -> u32 { self.stat.mode }
    fn uid(&self) -> u32 { self.stat.uid }
    fn gid(&self) -> u32 { self.stat.gid }
    fn size(&self) -> u64 { self.stat.size }
    fn mtime(&self) -> i64 { self.stat.mtime }
    fn nlink(&self) -> u32 { self.stat.nlink }
}
```

---

## Phase 3：sysroot 整合

**方式：建立 overlay sysroot（最實際的做法）**

```bash
# xv8-std/ 的目錄結構對應到 std 的原始碼佈局
xv8-std/
└── sysroot/
    └── lib/
        └── rustlib/
            └── targets/
                └── riscv64gc-unknown-none-elf/
                    └── libstd.a   # xv8-std-overlay 的編譯產物
```

```bash
# 建立自訂 toolchain
rustup toolchain link xv8 $(rustc +nightly --print sysroot)/..
rustup component add --toolchain xv8 rust-src

# 將 xv8-std-overlay 編譯進 sysroot
# 使用 `-Z build-std=std` 加上我們的 sysroot 作為 overlay
cargo +nightly build -Z build-std=std \
  --target riscv64gc-unknown-none-elf \
  --manifest-path xv8-std/xv8-std-overlay/Cargo.toml
```

另一種做法（在 `posix/tools/Cargo.toml` 中明確依賴 `xv8-std-overlay` 作為 `std` 的替代）需要 scholarship，但或許更簡單：直接把 `xv8-std-overlay` 包成一個與 `std` 名稱相容的 crate，讓 `posix/tools/` 在編譯到 xv8 時依賴它：

```toml
# posix/tools/Cargo.toml (xv8 target)
[dependencies]
std = { package = "xv8-std-overlay" }
```

這繞過了 compiler built-in std 的置換，直接在 crate 層面提供 `std` 作為普通 dependency。

---

## Stub 策略

功能**被呼叫時**（binary 真正用到）才 panic/stub return error，而非在宣告時就 panic。這樣多數 binary 能正常運作，只有用到 stub 功能的少數 binary 會在執行時失敗。

| 功能 | Stub 回傳 | 影響 binary |
|---|---|---|
| `SystemTime::now()` | 0 epoch | touch（可改用 atime/mtime workaround） |
| `thread::sleep` | 立即返回 | sleep（需實作） |
| `Command::spawn` | `io::ErrorKind::Unsupported` | sh, xargs, nohup（需實作 fork/exec） |
| `std::env::vars()` | empty | sh（需實作 getenv） |
| `chdir`（如果沒有 syscall） | error | sh |
| `MetadataExt::blocks()` | 0 | ls -l（顯示 0 blocks，幾可接受） |

---

## Phase 順序與驗收

| Phase | 內容 | 驗收標準 |
|---|---|---|
| 0 | target spec + cargo config | `cargo build --target riscv64gc...` 成功 |
| 1 | `xv8-libc`：io、fs、proc、mem syscall wrapper | `xv8-libc` crate 編譯成功 |
| 2.1 | `xv8-std-overlay`：io + fs | 20+ 個 text processing binary 能編譯 |
| 2.2 | `xv8-std-overlay`：path + env + ffi | `ls`、`cat` 等 binary 能編譯 |
| 2.3 | `xv8-std-overlay`：collections + time（stub）| 全部 100+ binary 能編譯 |
| 3 | sysroot 整合 + Command + thread | 全部能執行（stub 除外）|

---

## 參考資源

- [phil-opp: Implementing `std`](https://os.phil-opp.com/std/) — 從零實作 `std` 的教學
- [Redox OS `std` 实作](https://gitlab.redox-os.org/redox-os/redox) — 完整範例
- [rustix backend](https://github.com/bytecodealliance/rustix) — syscall wrapper 架構參考

---

## 總結

`posix/tools/` 用到的 `std` 極少，只需要實作：
- `std::io`（Read/Write/BufRead/Seek）→ 基於 `read`/`write`/`lseek` syscalls
- `std::fs`（File/read_to_string/metadata/read_dir）→ 基於 `open`/`read`/`stat`/`readdir` syscalls
- `std::path`（Path/PathBuf）→ 純指標包裝
- `std::env`（args/vars/current_dir）→ 基於 `exec` 參數和 `getcwd`
- `std::process`（Command/exit）→ 基於 `fork`/`exec`/`exit`
- `std::ffi`（CStr/CString）→ 指標包裝
- `std::collections`（HashMap）→ 直接用 `alloc`
- `std::time`（Duration/UNIX_EPOCH）→ stub `SystemTime::now()`
- `std::thread` → stub `sleep`
- `std::os::unix::fs`（MetadataExt/PermissionsExt）→ 基於 `fstat`

用不到的模組（`net`、`sync`、`thread::spawn`、`env::home_dir` 等）完全不碰。