# Panic — 恐慌處理

`panic.rs` 定義 xv8 使用者程式的恐慌（panic）處理器，當 Rust 程式遇到不可回復錯誤時觸發。

## 恐慌行為

xv8 的 panic handler 執行以下動作：

1. 輸出恐慌訊息（使用 `write` 系統呼叫到 stderr）
2. 輸出檔案名與行號（來自 `#[panic_handler]` 的 `PanicInfo`）
3. 呼叫 `exit(-1)` 終止當前行程
4. 若此為核心層的恐慌，則呼叫 `abort()` 暫停所有 CPU

## xv8 的適應

在 `riscv64gc-unknown-none-elf` 上，Rust 要求 `#[panic_handler]` 函式必須存在（無預設實作）。xv8-user-std 提供此 handler，讓使用者程式無需自行實作。

## 相關文件

- [process.md](./process.md) — 行程終止
- [lib.md](./lib.md) — xv8-user-std 總覽
