# Lib — XV8 使用者標準函式庫

`lib.rs` 是 `xv8-user-std` crate 的根模組，提供在 `riscv64gc-unknown-none-elf` 目標上運行的 Rust 標準函式庫覆蓋（std shim）。

## 設計目的

Rust 的 `std` 函式庫依賴 OS 支援（檔案系統、網路、行程管理等）。但在 `riscv64gc-unknown-none-elf` 目標上，`std` 無法編譯。xv8-user-std 透過以下模式提供相容層：

1. **重新匯出（reexport）**: 將真正的 `std` 模組重新匯出，但覆蓋特定平台相關功能
2. **系統呼叫橋接**: 為 xv8 系統呼叫提供 Rust 標準化的包裝
3. **條件編譯**: 在非 xv8 平台上委派給真實 `std`

## 支援的模組

`io`、`fs`、`net`、`path`、`env`、`process`、`sync`、`thread`、`time`、`ffi`、`os`、`panic`

所有模組會公開與 `std` 一致的 API。

## 相關文件

- [xv8-user-std.md](../xv8-user-std.md) — Crate 級文檔
- [lib.md](../../xv8-libc-compat/src/lib.md) — libc 相容層
- [raw.md](../../xv8-libc/src/raw.md) — 系統呼叫
