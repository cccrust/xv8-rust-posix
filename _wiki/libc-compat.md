# libc-compat（libc 相容層）

xv8-libc-compat 是 xv8rust 子專案中的一個 crate，讓 POSIX 工具可以透過統一的 `libc` 名稱來存取系統呼叫或 libc 函式。

## 設計動機

POSIX 工具（如 `cat`、`ls`、`grep`）依賴標準 C 庫函式（如 `open`、`read`、`printf`）。在傳統系統上，這些會呼叫 libc。xv8 的設計：

- **在主機（macOS/Linux）上**：直接使用系統的 libc
- **在 xv8（RISC-V）上**：提供最小化的 syscall wrapper

`posix/tools/Cargo.toml` 依賴 `libc`：

```toml
[dependencies]
libc = { path = "../xv8rust/xv8-libc-compat" }
```

這使同一份程式碼可以無縫編譯到兩個目標。

## 兩個實現路徑

`xv8-libc-compat/src/lib.rs` 使用條件編譯：

```rust
// 非 RISC-V 目標：委託給真正的 libc
#[cfg(not(target_arch = "riscv64"))]
pub use real_libc::*;

// RISC-V 目標：提供自己的實現
#[cfg(target_arch = "riscv64")]
mod riscv64_impl { ... }
```

## RISC-V 實現（RISC-V 模式）

在 RISC-V 上，`libc` 模組由 xv8-libc-compat 的 `riscv64_impl` 提供。

### 類型別名

定義與 C 相容的類型：

```rust
pub type c_int = i32;
pub type c_char = i8;
pub type c_uint = u32;
pub type size_t = usize;
pub type ssize_t = isize;
pub type pid_t = c_int;
pub type uid_t = u32;
pub type gid_t = u32;
pub type mode_t = u32;
```

### 結構體定義

必須與 C 布局完全一致（`#[repr(C)]`）：

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; NCCS],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct utsname {
    pub sysname: [c_char; 65],
    pub nodename: [c_char; 65],
    pub release: [c_char; 65],
    pub version: [c_char; 65],
    pub machine: [c_char; 65],
    pub domainname: [c_char; 65],
}
```

### 常數定義

定義與 POSIX 標準一致的常數：

```rust
pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;
pub const O_RDONLY: c_int = 0;
pub const O_WRONLY: c_int = 1;
pub const O_RDWR: c_int = 2;
pub const O_CREAT: c_int = 64;
pub const SIGTERM: c_int = 15;
pub const SIGCHLD: c_int = 20;
```

### 系統呼叫包裝

每個 libc 函式對應一個底層 syscall：

```rust
pub unsafe fn open(pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    syscall3(Syscall::Open, pathname as usize, flags as usize, mode as usize)
}

pub unsafe fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    syscall3(Syscall::Read, fd as usize, buf as usize, count as usize)
}

pub unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    syscall3(Syscall::Write, fd as usize, buf as usize, count as usize)
}
```

### Syscall 內嵌組語

syscall 是透過 `ecall` 指令發起的：

```rust
#[inline(always)]
fn syscall3(syscall: Syscall, a0: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") syscall as usize,
            in("a0") a0 as isize,
            in("a1") a1,
            in("a2") a2,
            lateout("a0") ret,
        );
    }
    ret
}
```

`a7` 放置系統呼叫號碼，`a0-a2` 放置參數。

### 錯誤處理

錯誤時 syscall 回傳負值，包裝函式轉換為 -1 並設定 errno：

```rust
#[inline]
fn check(ret: isize) -> c_int {
    if ret >= 0 {
        ret as c_int
    } else {
        -1
    }
}
```

### Stub 函式

有些函式在 xv8 上尚未完全實現：

```rust
pub unsafe fn getpwnam(_name: *const c_char) -> *mut passwd {
    core::ptr::null_mut()  // 尚未支援
}

pub unsafe fn shmctl<T>(_shmid: c_int, _cmd: c_int, _buf: *mut T) -> c_int {
    -1  // 尚未支援
}
```

這些 stub 讓程式碼能編譯，但執行時會返回錯誤。

## 主機實現（委託給 libc）

在非 RISC-V 平台上，`libc` simply re-exports from the real `libc` crate:

```rust
#[cfg(not(target_arch = "riscv64"))]
pub use real_libc::*;
```

這裡的 `real_libc` 是 Rust 官方維護的 `libc` crate，提供了所有標準 C 類型、 常數和函式的宣告。

## 與 xv8-user-std 的關係

xv8-libc-compat 專注於 C ABI 層面（類型、struct layout、syscall wrapper），而 [[xv8-std]] 專注於 Rust `std` 抽象層（`File`、`Read`、`Write` trait 等）。兩者結合，讓 Rust 的高層抽象能夠在 xv8 上運作。

## 常見的 libc 函式覆蓋

目前 xv8-libc-compat 提供的函式包括：

- **檔案操作**：`open`、`close`、`read`、`write`、`lseek`
- **程序管理**：`fork`、`exec`、`wait`、`exit`、`getpid`、`getppid`
- **目錄操作**：`getcwd`、`chdir`、`mkdir`、`rmdir`
- **環境變數**：`getenv`、`setenv`、`unsetenv`
- **時間**：`time`、`clock_gettime`、`nanosleep`
- **終端**：`tcgetattr`、`tcsetattr`、`isatty`
- **錯誤處理**：`errno`（執行緒本地的）

## 擴展 libc-compat

新增一個函式需要：

1. 在 `riscv64_impl` 中定義類型/常數（如需要）
2. 定義對應的 syscall 號碼
3. 實現包裝函式
4. 在主機端委託給真正的 libc

## 相關主題

- [[xv8-std]]：Rust std 抽象層
- [[Syscall]]：底層 syscall 機制
- [[Cross-Compilation]]：如何編譯到 RISC-V