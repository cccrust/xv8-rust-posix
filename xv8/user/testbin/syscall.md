# Syscall — 系統呼叫測試

`syscall` 測試驗證 xv8 的系統呼叫基礎設施，測試 52 個系統呼叫（由 xv8-libc 定義）的正確分派與執行。系統呼叫是使用者空間請求核心服務的唯一通道，遵循 RISC-V ecall 約定：編號存放於 a7 暫存器，參數依次在 a0–a5，回傳值在 a0。核心透過 `syscall_handler` 分發表根據編號分派到對應的處理函式。

## 相關文件

- [syscall.md](../../kernel/src/syscall.md) — 核心系統呼叫處理
- [raw.md](../../../xv8rust/xv8-libc/src/raw.md) — 系統呼叫包裝
- [abi.md](../../kernel/src/abi.md) — ABI 定義
