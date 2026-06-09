# Error — 核心錯誤處理

## 概述

xv8 核心採用 Rust 的 `Result<T, E>` 模式處理錯誤，`SysError` 列舉型別對應 POSIX 錯誤碼（errno）。這種設計讓核心內部的錯誤處理與使用者空間的 errno 機制無縫銜接。

## SysError 列舉

`SysError` 的每個 variant 對應 Linux 定義的 errno 常數（如 `EINVAL`、`ENOMEM`、`EACCES`、`ENOENT`）。核心函式回傳 `SysError` 時，trap 處理器將其轉為使用者空間的負數回傳值，libc 層再轉為全域 `errno`。

## 輔助巨集

### `try_log!`
Rust 的 `?` 運算子會提早回傳錯誤，但在核心中我們常需要在錯誤發生時記錄診斷資訊。`try_log!` 巨集包裝 `?`，在回傳前透過 `printf` 輸出錯誤上下文。

### `err!`
`err!` 巨集簡化 `Err(SysError::SomeVariant)` 的寫法，減少樣板程式碼。

## 錯誤處理哲學

Unix 核心將錯誤碼作為回傳值傳遞，而非例外（exception）。這種設計強調顯式錯誤檢查——呼叫者必須檢查每個系統呼叫的回傳值。xv8 延續此傳統，但透過 Rust 的型別系統在編譯期強制錯誤處理：`Result` 型別要求呼叫者必須處理 `Err` 分支。

## 相關文件

- [abi.md](./abi.md) — ABI 定義與系統呼叫介面
- [trap.md](./trap.md) — 異常與系統呼叫的陷阱處理
