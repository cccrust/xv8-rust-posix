# xv8 標準覆蓋層 — xv8rust/

xv8rust 提供 xv8 作業系統的 Rust 標準函式庫覆蓋層 (std overlay)，讓一般 Rust 程式碼（使用 `std::net`、`std::io`、`std::time` 等）能在 xv8 的 RISC-V 環境下無痛編譯執行。

## 背景

xv8 是一個從零打造的 RISC-V Unix-like 作業系統，其使用者空間沒有標準的 libc 或 Rust std。xv8rust 的目標是提供一個 `#![no_std]` 的相容層，讓既有 Rust 生態系的非同步執行緒、HTTP、網路等函式庫能在 xv8 上執行。

## 架構

```
xv8rust/
├── Cargo.toml          # 工作空間 (9 個 crate)
├── xv8-libc/           # 52 個原始系統呼叫包裝器 (RISC-V inline asm)
├── xv8-libc-compat/    # libc 相容性墊片 (posix tools 使用)
├── xv8-user-std/       # 使用者空間 std 覆蓋層 (io, net, sync, time, fs)
├── xv8-async/          # 非同步執行緒 + epoll reactor
├── xv8-tokio-compat/   # Tokio 相容層 (AsyncRead/Write, TcpStream, runtime)
├── xv8-http/           # HTTP/1.1 類型 + 解析器 (#![no_std])
├── xv8-router/         # 輕量級 axum-like Router (#![no_std])
├── xv8-net/            # #![no_std] std::net 相容層
├── xv8-axum-smoke/     # 整合測試 (axum 相容性驗證)
└── crossterm/          # 移植的 crossterm 0.29.0 (vi/vim 用)
```

## Crate 說明

| Crate | 路徑 | 用途 |
|-------|------|------|
| `xv8-libc` | `xv8-libc/` | 52 個原始系統呼叫 (RISC-V inline asm)，包含 `open`、`read`、`write`、`mmap`、`sched_yield` 等 |
| `xv8-libc-compat` | `xv8-libc-compat/` | 在 RISC-V 上提供 `libc` crate 相容介面；在 host 上委派給真實 libc |
| `xv8-user-std` | `xv8-user-std/` | 對應 Rust std 的模組：`std::io`、`std::net`、`std::sync`、`std::time`、`std::fs`、`std::path`、`std::process`、`std::env`、`std::thread`、`std::ffi`、`std::panic` |
| `xv8-async` | `xv8-async/` | 基於 epoll 的非同步執行緒 (reactor 模式)，提供 `AsyncRead`/`AsyncWrite` |
| `xv8-tokio-compat` | `xv8-tokio-compat/` | Tokio 相容層，讓 `tokio::net::TcpStream`、`tokio::runtime::Runtime` 等可在 xv8 使用 |
| `xv8-http` | `xv8-http/` | 純 `#![no_std]` 的 HTTP/1.1 封包解析器，無 alloc 依賴 |
| `xv8-router` | `xv8-router/` | 類似 axum 的 Router，支援 `#[derive(Route)]` 模式的路由匹配 |
| `xv8-net` | `xv8-net/` | `#![no_std]` 的 std::net 相容層 |
| `crossterm` | `crossterm/` | 移植的 crossterm v0.29.0，提供終端機控制 (vi/vim 使用) |

## 核心設計原理

### 系統呼叫包裝

`xv8-libc` 使用 RISC-V `ecall` 指令直接呼叫核心，每個系統呼叫包裝為一個內聯組合語言函數：

```rust
// xv8-libc/src/raw.rs 中的典型範例
pub fn open(path: *const u8, flags: usize, mode: usize) -> isize {
    let ret: isize;
    unsafe { asm!("ecall", in("a7") 16, in("a0") path, in("a1") flags, in("a2") mode, lateout("a0") ret); }
    ret
}
```

### 非同步執行緒

`xv8-async` 使用核心的 `epoll` 機制實作 reactor。由於 xv8 沒有 `mio` 或 `tokio` 原生支援，reactor 直接包裝核心的 `epoll_create1`、`epoll_ctl`、`epoll_wait` 系統呼叫。

## 相容性限制

- **CSR 陷阱**: `riscv64gc-unknown-none-elf` 會在 `core::sync::atomic` 操作時產生 `csrr sstatus`，這會在使用者模式下觸發陷阱。應避免在 RISC-V 使用者程式碼中使用原子操作
- **Resolver 不一致**: `net/` 使用 resolver `"2"`，`posix/` 和 `xv8rust/` 使用 `"3"`
- **tokio 修補**: `Cargo.toml` 中 tokio 被修補 (patch) 到本機路徑，僅在原始開發機器上可用

## 測試

```bash
cd xv8 && ./test.sh  # 包含 async + httpepoll + axum 測試
```

- `_async` 測試：驗證 async runtime + epoll reactor 基本功能
- `_httpepoll` 測試：驗證 HTTP 伺服器在 async runtime 上的運作
- `_axum` 測試：驗證 xv8-router 與 tokio-compat 的整合

## 相關文件

- [Wiki: xv8-std](../_wiki/xv8-std.md)
- [Wiki: Rust no_std](../_wiki/Rust-no_std.md)
- [xv8 核心文件](../xv8/kernel/src/)
- [計劃版本記錄](_doc/)
