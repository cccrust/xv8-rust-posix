# XV8-Libc-Compat — libc 相容性層

`lib.rs` 是 xv8-libc-compat crate 的根模組，提供與標準 Rust `libc` crate 相容的型別與函式簽名，底層委派給 xv8-libc。

## 委派模式

此 crate 採用委派（delegation）設計模式：

```
使用者程式 → 標準 libc API → xv8-libc-compat → xv8-libc → ecall → 核心
```

標準 Rust 程式碼通常透過 `libc` crate 訪問系統呼叫。xv8-libc-compat 實作相同的函式簽名與型別（如 `c_int`、`size_t`、`struct stat`），將呼叫轉發到 xv8-libc 的原始包裝。

## 在 host 上的行為

當編譯目標不是 `riscv64gc-unknown-none-elf` 時（例如在 macOS 上執行單元測試），xv8-libc-compat 自動委派給真正的系統 `libc`。這透過 `cfg(target_arch = "riscv64")` 條件編譯實現。

## 支援的型別

- `c_int`, `c_uint`, `c_long`, `c_ulong`, `size_t`, `ssize_t`, `off_t`
- `struct stat`, `struct sockaddr`, `struct sockaddr_in`, `struct timeval`
- errno 常數（EINVAL, ENOMEM, EACCES, ENOENT 等）

## 相關文件

- [xv8-libc-compat.md](../xv8-libc-compat.md) — Crate 級文檔
- [raw.md](../../xv8-libc/src/raw.md) — 原始系統呼叫
- [args.md](../../xv8-libc/src/args.md) — 系統呼叫參數
