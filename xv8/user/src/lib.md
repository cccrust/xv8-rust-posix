# lib — 使用者程式函式庫

xv8 使用者程式的核心函式庫，提供 no_std 環境所需的各種功能。

## 模組結構

```rust
#![no_std]

mod io;      // I/O traits and macros
mod args;    // Command-line argument parsing
mod line;    // Line editor
mod syscall; // System call wrappers

pub use kernel::abi::*;  // Kernel ABI (errors, types)
pub use args::*;
pub use io::{Read, Stderr, Stdin, Stdout, Write};
pub use line::LineEditor;
pub use syscall::*;
```

## 入口點

```rust
unsafe extern "Rust" {
    fn main(args: Args);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
fn _start() -> ! {
    unsafe {
        let args = Args::from_stack();
        main(args);
        exit(0);
    }
}
```

流程：
1. 核心跳转到 `.text.entry` 段
2. `_start()` 從 stack 取得參數
3. 呼叫 `main(args)`
4. `main` 返回後呼叫 `exit(0)`

## Panic 處理

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("! {}", info);
    exit(1)
}
```

Panic 時輸出訊息並以退出碼 1 終止。

## 主要匯出

| 匯出 | 說明 |
|------|------|
| `Args` | 命令列參數 |
| `Fd` | 檔案描述符 |
| `Read/Write` | I/O trait |
| `Stdin/Stdout/Stderr` | 標準串流 |
| `print!/println!/eprint!/eprintln!` | 列印巨集 |
| `fork/exec/wait/exit` | 程序管理 |
| `open/read/write/close` | 檔案操作 |
| `socket/send/receive` | 網路 |
| `sleep/uptime` | 時間 |
| `SysError` | 錯誤類型 |

## 與核心的整合

- 依賴 `kernel` crate 提供的 ABI
- 使用 RISC-V ecall 指令發起系統呼叫
- 透過 `abi` module 共享類型定義

## 與 xv8rust 的差異

`user` crate：
- 核心模式（無標準庫）
- 直接系統呼叫
- 輕量級

`xv8rust`：
- 使用者模式（有標準庫）
- POSIX 相容包裝
- 豐富功能

## 相關主題

- [[syscall]]：系統呼叫包裝
- [[args]]：參數解析
- [[io]]：I/O 功能