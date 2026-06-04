# xv8-rust-posix

Xv8-rust-posix 是基於 xv6-riscv 延伸的全 Rust 作業系統專案，包含 RISC-V 核心、POSIX 使用者工具集、自製標準函式庫以及網路工具。

## 子專案

| 專案 | 目錄 | 說明 | 狀態 |
|------|------|------|------|
| xv8 OS | `xv8/` | RISC-V Unix-like 核心（QEMU virt），110+ 系統呼叫、網路堆疊、日誌檔案系統 | 11/11 測試通過 |
| POSIX 工具 | `posix/` | 124 個 POSIX.1-2008 相容工具 + 完整 shell（管線、重導、工作控制） | 33/33 shell + 21/21 core 測試通過 |
| xv8 std | `xv8rust/` | 跨平臺標準函式庫：xv8-user-std（riscv64 no_std）、xv8-libc（系統呼叫層）、crossterm（終端操作） | riscv64 交叉編譯成功 |
| 網路工具 | `net/` | 13 個主機端網路工具（ping、dns、tcp、ntp、whois 等） | 煙霧測試通過 |

## 快速開始

```bash
./shell.sh              # 建置 + 啟動 POSIX shell（提示符: posix>）
./test.sh               # 完整測試：POSIX 測試 + xv8 交叉編譯 + QEMU 整合測試
cd posix && ./test.sh   # POSIX shell + core tools 測試
cd xv8   && ./test.sh   # QEMU 整合測試（10 項）
cd net   && ./test.sh   # 網路工具煙霧測試
```

## 版本歷程

| 版本 | 主要內容 | 文件 |
|------|---------|------|
| v1.3 | Shell 語法修正（bash/sh 三項 bug） | `_doc/v1.3.md` |
| v1.4 | ls -R、diff LCS 驗證 | `_doc/v1.4.md` |
| v1.5 | riscv64 shell 相容 + tcgetattr/tcsetattr | `_doc/v1.5.md` |
| v1.6 | riscv64 crossterm 編譯（xv8-user-std 擴充） | `_doc/v1.6.md` |
| v1.7 | ex、fc 工具實作 | `_doc/v1.7.md` |
| v1.8 | iconv 編碼擴充（支援 15+ 編碼） | `_doc/v1.8.md` |
| v2.0+ | 規劃中（見 `_doc/todo2.md`） | `_doc/todo2.md` |

## 架構說明

- **無根 Cargo.toml** — 每個子專案各自獨立 workspace
- **Rustc wrapper** (`.cargo/rustc-wrapper.sh`) — 為 riscv64 目標注入 `#![no_main]` + `#[no_mangle]`
- **libc → xv8-libc-compat** — 主機端委託給真實 `libc`，riscv64 使用自製系統呼叫層
- **Crossterm 改編版** — 支援 riscv64 編譯（終端操作子集），事件系統除外
- **工具鏈**：nightly + `riscv64gc-unknown-none-elf`
- **xv8 釋出設定**：`lto = true`、`strip = true`、`codegen-units = 1`

## 建置

```bash
# 主機端建置（macOS/Linux）
cd posix && cargo build --release

# riscv64 交叉編譯
cargo build --release --manifest-path posix/Cargo.toml --target riscv64gc-unknown-none-elf --no-default-features
cargo build --release --manifest-path xv8rust/Cargo.toml --target riscv64gc-unknown-none-elf

# 含 crossterm（vi/vim）的 riscv64 建置
cargo build --release -p tools --target riscv64gc-unknown-none-elf --features crossterm

# 核心 + QEMU 執行
cd xv8 && make qemu
```

## 授權

本專案衍生自 xv6-riscv（MIT 授權），延續 MIT 授權。