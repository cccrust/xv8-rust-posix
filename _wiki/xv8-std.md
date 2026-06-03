# xv8-std（std 覆寫層）

xv8rust 是 xv8-rust-posix 工作區中的子專案，負責提供 Rust `std` 標準庫的實作子集，讓原本為主機設計的 POSIX 工具能夠編譯到 RISC-V 目標。

## 設計動機

Rust 的 `std` 標準庫依賴作業系統提供的功能（檔案、網路、執行緒等）。在 `no_std` 環境（無標準庫）中，這些都不可用。

xv8rust 的設計目標：
- 讓 `std::fs::File`、`std::io::Read` 等抽象能在 xv8 上工作
- 讓同一份 Rust 程式碼可以編譯到主機（macOS/Linux/x86）和 xv8（RISC-V）
- 提供最小必要的 `std` 功能子集，不追求完整相容

## 專案結構

```
xv8rust/
├── Cargo.toml          # 工作區清單
├── xv8-user-std/       # Rust std 覆寫層
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # 核心實作
│       ├── fs.rs        # 檔案系統抽象
│       ├── io.rs        # I/O traits
│       ├── net.rs       # 網路抽象
│       ├── process.rs   # 程序/執行緒
│       ├── time.rs      # 時間相關
│       ├── env.rs       # 環境變數
│       ├── path.rs      // 路徑處理
│       ├── thread.rs    # 執行緒
│       ├── runtime.rs   # 執行期輔助
│       └── os/          # 平台特定
├── xv8-libc-compat/    # libc 相容層
│   └── src/lib.rs
└── crossterm/          # 終端控制（vendored）
    └── src/
```

## xv8-user-std 的核心實作

### 條件編譯

```rust
// xv8-user-std/src/lib.rs
#![no_std]
#![feature(libstd_sys_internals)]

extern crate libc;
extern crate alloc;
```

使用 `#![no_std]` 屬性，完全擺脫標準庫的依賴。`libc` crate 來自 xv8-libc-compat。

### alloc crate

需要動態記憶體分配時，使用 `alloc` crate（恐慌時終止）：

```rust
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
```

### 符號別名

Rust 編譯器內部使用某些符號名稱（如 `__rust_syscall_base`）。xv8-user-std 通過 `build.rs` 在編譯時注入必要的符號別名。

## I/O 抽象

### Read/Write traits

```rust
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
}
```

這些 traits 定義了標準的輸入輸出介面。`File`、`TcpStream` 等都實作了這些 traits。

## 檔案系統抽象

```rust
pub struct File {
    fd: RawFd,
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(n as usize) }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = unsafe { libc::write(self.fd, buf.as_ptr() as *const _, buf.len()) };
        if n < 0 { Err(io::Error::last_os_error()) } else { Ok(n as usize) }
    }
}
```

## 錯誤處理

`std::io::Error` 和 `Result` 類型在 xv8-user-std 中重新定義：

```rust
pub struct Error { /* ... */ }

pub type Result<T> = Result<T, Error>;

impl Error {
    pub fn last_os_error() -> Error { /* ... */ }
    pub fn new(kind: ErrorKind, msg: &str) -> Error { /* ... */ }
}
```

## 環境變數

```rust
pub fn getenv(key: &str) -> Option<String> {
    let c_key = CString::new(key).ok()?;
    let value = unsafe { libc::getenv(c_key.as_ptr()) }?;
    Some(CStr::from_ptr(value).to_string_lossy().into_owned())
}
```

## 路徑處理

`std::path` 的子集實作：

```rust
pub struct Path { /* ... */ }
pub struct PathBuf { /* ... */ }

impl Path {
    pub fn new(s: &str) -> &Path { /* ... */ }
    pub fn to_string_lossy(&self) -> Cow<str> { /* ... */ }
}
```

## 格式化輸出

xv8-user-std 提供了基本的 `println!` 和 `format!` 巨集。這些在 `alloc` 的 `fmt` 模組基礎上構建。

## 與 libc-compat 的關係

xv8-user-std 建構在 [[libc-compat]] 之上：

- 使用 libc-compat 提供的類型和常數
- 呼叫 libc-compat 的函式包裝
- 提供更高層的 Rust 抽象

```
應用程式
   ↓
std traits (Read, Write, etc.)
   ↓
xv8-user-std (File, TcpStream, etc.)
   ↓
libc-compat (syscall wrappers)
   ↓
kernel syscall (ecall)
```

## crossterm 整合

xv8rust 包含了 crossterm 的 vendored 版本（0.29.0），用於 vi/vim 等終端應用。在 RISC-V 上編譯時使用 `--no-default-features` 排除主機特定功能。

## 編譯目標

xv8-user-std 編譯到：
- `riscv64gc-unknown-none-elf`：xv8 目標
- （理論上）其他 `no_std` 環境

## 限制與未來擴展

目前的 xv8-user-std 是一個最小化實作：
- 不支援網路（`std::net` 尚未完全實作）
- `RwLock`、`Mutex` 等同步原語可能需要調整
- `std::thread` 只有基本功能

## 相關主題

- [[libc-compat]]：C ABI 層面
- [[Cross-Compilation]]：如何編譯到 RISC-V
- [[Rust-no_std]]：no_std 程式設計概念
- [[Shell]]：使用 xv8-user-std 的 shell