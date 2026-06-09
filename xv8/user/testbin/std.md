# Std — 標準函式庫測試

`std` 測試驗證 xv8 的標準函式庫覆蓋（xv8-user-std）在 RISC-V 目標上的正確性。測試涵蓋 `std::io`、`std::fs`、`std::net`、`std::sync`、`std::time`、`std::process`、`std::env`、`std::path`、`std::ffi` 等模組的橋接實作。確認這些模組在 `riscv64gc-unknown-none-elf` 無作業系統目標上正確代理到 xv8 系統呼叫。

## 相關文件

- [xv8-user-std/mod.md](../../../xv8rust/xv8-user-std/src/lib.md) — xv8 std 總覽
- [lib.md](../../../xv8rust/xv8-libc-compat/src/lib.md) — libc 相容層
