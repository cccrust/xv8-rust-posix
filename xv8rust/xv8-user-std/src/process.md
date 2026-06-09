# Process — 行程管理

`process.rs` 實作 `std::process` 模組，提供行程建立（`Command`）、程序退出（`exit`）與引數處理。

## Command

`std::process::Command` 在 xv8 上透過以下系統呼叫實作：

1. **Command::new/spawn**: `fork()` 建立子行程，`execve()` 執行目標程式
2. **Output/Status**: `waitpid()` 等待子行程結束並收集狀態
3. **Pipe 重定向**: `pipe()` + `dup2()` 連接 stdin/stdout/stderr

## xv8 的適應

在 xv8 上，`exit` 直接使用 `exit` 系統呼叫終止行程。由於 xv8 核心在行程終止時不寫入 `/proc/self/status` 等檔案，部分程序狀態查詢行為受限。

## 相關文件

- [env.md](./env.md) — 環境變數
- [proc.md](../../kernel/src/proc.md) — 核心行程管理
- [exec.md](../../kernel/src/exec.md) — Exec 載入器
