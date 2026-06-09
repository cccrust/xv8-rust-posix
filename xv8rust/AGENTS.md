# xv8rust — AI 輔助開發指南

## 專案概述

xv8rust 是 xv8 作業系統的 Rust 標準函式庫覆蓋層 (std overlay)，9 個獨立 crate 組成，無根層級 Cargo.toml。

| Crate | 路徑 | 類型 |
|-------|------|------|
| xv8-libc | `xv8-libc/` | 52 個原始 syscall (RISC-V asm) |
| xv8-libc-compat | `xv8-libc-compat/` | libc 相容墊片 |
| xv8-user-std | `xv8-user-std/` | std 覆蓋層 (io, net, sync, time, fs) |
| xv8-async | `xv8-async/` | 非同步執行緒 + epoll reactor |
| xv8-tokio-compat | `xv8-tokio-compat/` | Tokio 相容層 |
| xv8-http | `xv8-http/` | HTTP/1.1 類型 + 解析器 (`#![no_std]`) |
| xv8-router | `xv8-router/` | 輕量級 axum-like Router (`#![no_std]`) |
| xv8-net | `xv8-net/` | std::net 相容層 (`#![no_std]`) |
| crossterm | `crossterm/` | 移植 crossterm 0.29.0 |

## 常用命令

```bash
# 建置全部
cargo build --release

# 建置單一 crate
cargo build --release -p xv8-user-std

# RISC-V 交叉編譯
cargo build --release --target riscv64gc-unknown-none-elf -p xv8-user-std

# 測試 (透過 xv8 QEMU)
cd ../xv8 && cargo build --release --package user && ./test_core.sh
```

## 關鍵限制

- **無根 Cargo.toml** — 每個 crate 獨立工作；不可使用 `cargo test` 直接在本機測試
- **tokio 修補路徑** — `Cargo.toml` 中 tokio 指向 `/Users/Shared/...`，僅原開發機器可用
- **CSR 陷阱** — `riscv64gc-unknown-none-elf` 目標的 `core::sync::atomic` 會產生 `csrr sstatus`，在使用者模式下觸發陷阱。避免使用 `Arc`、`AtomicBool` 等原子操作
- **`#![no_std]`** — xv8-http 和 xv8-router 不使用 alloc，需注意堆疊配置
- **Resolver 不一致** — `xv8rust/` 使用 resolver `"3"`

## 架構注意事項

- `xv8-libc` 的所有函數都是 `unsafe`，呼叫端需自行保證正確性
- `xv8-user-std` 對應 Rust `std` 的模組名稱，提供 `std::io`、`std::net` 等介面
- `xv8-async/reactor.rs` 直接包裝核心的 epoll 系統呼叫
- `xv8-tokio-compat` 實作 `tokio::io::AsyncRead`/`AsyncWrite` 等 trait，讓 tokio 生態系可在 xv8 使用

## 相關文件

- [README.md](README.md)
- [計劃版本記錄](_doc/)
- [xv8 AGENTS.md](../xv8/AGENTS.md)
