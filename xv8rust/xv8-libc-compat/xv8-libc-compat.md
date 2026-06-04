# xv8-libc-compat

POSIX libc 相容層，在主機（macOS/Linux）上委托給真正的 libc，在 RISC-V 目標上提供系統呼叫包裝。

## 設計

```toml
[target.'cfg(not(target_arch = "riscv64"))'.dependencies]
real_libc = { package = "libc", version = "0.2" }
```

在非 RISC-V 平台上，直接使用標準 `libc` crate。

## RISC-V 系統呼叫

在 RISC-V 目標上，提供 syscall 包裝：

```rust
#[cfg(target_arch = "riscv64")]
pub fn write(fd: i32, buf: *const u8, len: usize) -> isize {
    let ret;
    unsafe {
        asm!(
            "ecall",
            inout("a0") fd => ret,
            in("a1") buf,
            in("a2") len,
            in("a7") 64, // SYS_write
        );
    }
    ret
}
```

## 跨平台統一介面

無論底層是真正的 libc 還是自訂 syscall，包裝後的 API 保持一致。

## 與 posix/tools 的關係

`posix/tools/Cargo.toml`：
```toml
libc = { package = "xv8-libc-compat", path = "../../xv8rust/xv8-libc-compat" }
```

POSIX 工具通過此包裝使用系統呼叫。

## 支援的 syscall

- 檔案操作：read, write, open, close, lseek
- 程序管理：fork, exec, wait, exit
- 記憶體：brk, mmap
- 網路：socket, connect, send, recv
- 時間：gettimeofday, clock_gettime

## 主機模式

在 macOS/Linux 上運行時：
- `libc::read()` → 真正的系統呼叫
- `libc::write()` → 真正的系統呼叫

## xv8 目標模式

在 RISC-V 上運行時：
- 所有操作通過 ECALL 指令触發 supervisor 模式
- kernel 提供實際的系統服務

## 底層機制

| 平台 | 系統呼叫方式 |
|------|-------------|
| macOS | libc wrapper → BSD kernel |
| Linux | libc wrapper → glibc → kernel |
| RISC-V/xv8 | ECALL → kernel |

## 安全性

此包裝層不增加安全性風險，只是 API 轉發。

## 相關套件

- `xv8-libc`：核心實作
- `xv8-user-std`：std overlay
- `libc`：主機依賴