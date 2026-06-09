# XV8-Libc — 系統呼叫函式庫

`lib.rs` 是 xv8-libc crate 的根模組，定義了共 52 個系統呼叫的 Rust 綁定。此函式庫直接使用 RISC-V `ecall` 指令陷入核心，無需依賴任何標準函式庫。

## 設計動機

`riscv64gc-unknown-none-elf` 目標不提供標準 C 函式庫。xv8-libc 填補此空缺，提供使用者程式可直接使用的系統呼叫包裝，類似於 `libc` crate 但針對 xv8 核心的 ABI 自訂。

## 組織結構

- **raw.rs**: 52 個內聯組合語言的系統呼叫包裝
- **args.rs**: 參數打包工具
- **lib.rs**: 公開匯出所有系統呼叫，並定義基礎型別（如 `c_int`、`size_t`）

## 公開介面

所有系統呼叫函式公開為 `unsafe` 函式，因為系統呼叫可變更行程的任意狀態。呼叫者需確保傳遞的指標有效、buf 長度正確。

## 相關文件

- [raw.md](./raw.md) — 系統呼叫包裝詳解
- [args.md](./args.md) — 參數打包
- [xv8-libc.md](../xv8-libc.md) — Crate 級文檔
