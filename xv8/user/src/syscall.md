# syscall — 系統呼叫包裝

xv8 使用者程式的系統呼叫介面，包裝 RISC-V ecall 指令。

## 模組結構

```
syscall/
├── raw/      內嵌組言層，直接發起 ecall
└── (其他)    Rust 包裝層，錯誤處理
```

## Raw Syscall 層

使用 `ecall` 指令發起系統呼叫：

```rust
#[inline(always)]
fn syscall1(syscall: Syscall, a0: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall as usize,  // 系統呼叫號
            inlateout("a0") a0 as isize => ret,
        )
    }
    ret
}
```

| 參數數 | 使用暫存器 |
|--------|-----------|
| 0 | a0 |
| 1 | a0, a1 |
| 2 | a0, a1, a2 |
| 3 | a0, a1, a2, a3 |
| 4 | a0, a1, a2, a3, a4 |
| 5 | a0, a1, a2, a3, a4, a5 |
| 6 | a0..a5 + a6 |

返回値在 a0，負值表示錯誤。

## Fd 結構

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fd(usize);

impl Fd {
    pub const STDIN: Fd = Fd(0);
    pub const STDOUT: Fd = Fd(1);
    pub const STDERR: Fd = Fd(2);

    pub fn as_raw(&self) -> usize { self.0 }
    pub fn from_raw(raw: usize) -> Self { Fd(raw) }
}
```

## 錯誤處理

```rust
#[inline(always)]
fn check(ret: isize) -> Result<usize, SysError> {
    if ret >= 0 {
        Ok(ret as usize)
    } else {
        Err(SysError::from_code((-ret) as u16))
    }
}
```

syscall 返回負值時，取絕對值作為錯誤碼。

## 檔案操作

```rust
pub fn open(path: &str, flags: usize) -> Result<Fd, SysError> {
    let cpath = validate_path(path)?;
    check(raw::open(cpath.as_ptr(), flags)).map(Fd)
}

pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize, SysError> {
    check(raw::read(fd.as_raw(), buf.as_mut_ptr(), buf.len()))
}

pub fn write(fd: Fd, buf: &[u8]) -> Result<usize, SysError> {
    check(raw::write(fd.as_raw(), buf.as_ptr(), buf.len()))
}

pub fn close(fd: Fd) -> Result<(), SysError> {
    check_unit(raw::close(fd.as_raw()))
}
```

## 程序管理

```rust
pub fn fork() -> Result<usize, SysError> {
    check(raw::fork())
}

pub fn exit(code: usize) -> ! {
    raw::exit(code)
}

pub fn wait(status: &mut usize) -> Result<usize, SysError> {
    check(raw::wait(status as *mut usize))
}

pub fn exec(path: &str, argv: &[&str]) -> SysError {
    // 建構參數並呼叫 raw::exec
    // exec 成功時不回返，失敗才回返錯誤
}
```

## 網路操作

```rust
pub fn socket(port: u16) -> Result<Fd, SysError> {
    check(raw::socket(port)).map(Fd)
}

pub fn send(fd: Fd, buf: &[u8], dest_ip: &[u8; 4], dest_port: u16) -> Result<usize, SysError> {
    check(raw::send(fd.as_raw(), buf.as_ptr(), buf.len(), dest_ip.as_ptr(), dest_port))
}

pub fn receive(fd: Fd, buf: &mut [u8], src_ip: &mut [u8; 4], src_port: &mut u16) -> Result<usize, SysError> {
    check(raw::receive(fd.as_raw(), buf.as_mut_ptr(), buf.len(), src_ip.as_mut_ptr(), src_port as *mut u16))
}
```

## TCP 操作

```rust
pub fn tcp_socket() -> Result<Fd, SysError>
pub fn tcp_bind(fd: Fd, port: u16) -> Result<(), SysError>
pub fn tcp_listen(fd: Fd) -> Result<(), SysError>
pub fn tcp_accept(fd: Fd) -> Result<Fd, SysError>
pub fn tcp_connect(fd: Fd, dest_ip: &[u8; 4], dest_port: u16) -> Result<(), SysError>
pub fn tcp_send(fd: Fd, buf: &[u8]) -> Result<usize, SysError>
pub fn tcp_recv(fd: Fd, buf: &mut [u8]) -> Result<usize, SysError>
```

## 訊號操作

```rust
pub fn sigaction(sig: usize, act: Option<&SigAction>, oldact: Option<&mut SigAction>) -> Result<(), SysError>
pub fn sigprocmask(how: i32, set: Option<u32>) -> Result<u32, SysError>
pub fn sigpending() -> Result<u32, SysError>
pub fn sigsuspend(mask: u32) -> Result<(), SysError>
pub fn sigreturn(ctx: *const u8) -> !
```

## 記憶體操作

```rust
pub fn sbrk(n: isize) -> Result<usize, SysError>
pub fn mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: isize, offset: usize) -> Result<usize, SysError>
pub fn munmap(addr: usize, length: usize) -> Result<(), SysError>
pub fn mprotect(addr: usize, length: usize, prot: usize) -> Result<(), SysError>
```

## 路徑驗證

```rust
struct Path<'a>(&'a str);

impl<'a> Path<'a> {
    fn new(s: &'a str) -> Result<Self, SysError> {
        if s.len() >= MAXPATH || s.bytes().any(|b| b == 0) {
            return Err(SysError::NameTooLong);
        }
        Ok(Self(s))
    }

    fn as_cpath(&self) -> [u8; MAXPATH] {
        let mut buf = [0u8; MAXPATH];
        buf[..self.0.len()].copy_from_slice(self.0.as_bytes());
        buf
    }
}
```

## Iovec

```rust
#[repr(C)]
pub struct Iovec {
    pub iov_base: *mut u8,
    pub iov_len: usize,
}

pub fn readv(fd: Fd, iovs: &mut [Iovec]) -> Result<usize, SysError>
pub fn writev(fd: Fd, iovs: &[Iovec]) -> Result<usize, SysError>
```

## 使用範例

```rust
use user::{open, read, close, Fd};

let fd = open("file.txt", OpenFlag::READ_ONLY)?;
let mut buf = [0u8; 512];
let n = fd.read(&mut buf)?;
close(fd)?;
```

## 與核心的整合

- 系統呼叫號定義在 `kernel::abi::Syscall`
- 核心透過 a7 暫存器取得系統呼叫號
- 核心透過 a0-a5 取得參數
- 返回値放在 a0

## 相關主題

- [[lib]]：入口點
- [[args]]：參數處理