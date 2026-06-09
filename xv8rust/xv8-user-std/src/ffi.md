# Ffi — 外部函式介面

`ffi.rs` 實作 `std::ffi` 模組的子集，提供 C 字串（`CStr`、`CString`）與 OS 字串（`OsStr`、`OsString`）的處理。

## xv8 的適應

在 xv8 的 `riscv64gc-unknown-none-elf` 目標上，FFI 類型與 host 平台一致——C 字串總是以 null 終止的位元組序列。由於 xv8 使用者程式不連結標準 C 函式庫，FFI 的主要用途是與核心系統呼叫互動（核心期望 null 終止的字串指標）。

## 相關文件

- [lib.md](./lib.md) — xv8-user-std 總覽
- [path.md](./path.md) — 路徑處理
