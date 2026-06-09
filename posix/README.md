# POSIX 工具集 — posix/

本專案為 xv8 作業系統提供完整的 POSIX 使用者空間工具集，包含 124 個標準 POSIX 工具與一個功能完整的 shell。

## 背景

POSIX (Portable Operating System Interface) 是 IEEE 制定的作業系統介面標準，定義了 shell、公用程式 (utilities) 與系統呼叫的行為。本專案實作 POSIX.1-2017 中定義的 124 個工具，包括檔案操作、文字處理、程式開發、系統管理等類別。

xv8-posix 的核心特色在於：同一份 Rust 程式碼可同時編譯為 **RISC-V 64 位元 (riscv64gc-unknown-none-elf)** 與 **主機 (Host) x86-64/ARM64** 二進位檔案。這得益於 `rustc-wrapper.sh` 在編譯 RISC-V 目標時自動注入 `#![no_main]` 與 `#[no_mangle]`，讓 POSIX 工具在兩個平台上都能無痛執行。

## 架構

```
posix/
├── Cargo.toml       # 工作空間 (workspace)，僅包含 tools crate
├── AGENTS.md         # AI 輔助開發指南
├── test.sh           # 測試執行腳本 (33 shell + 21 core tests)
├── .cargo/
│   └── config.toml   # RISC-V 目標配置 (測試時 stash/restore)
└── tools/            # 單一 crate
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs    # 共用函式庫 (signal, terminal, 路徑處理等)
    │   └── bin/      # 144 個二進位檔案 (124 工具 + 測試)
    └── tests/        # 測試腳本
        ├── basic.rs
        ├── test_sh_basic.sh
        ├── test_tools_core.sh
        └── test_v2.0.sh
```

## 工具分類

| 類別 | 數量 | 範例 |
|------|------|------|
| 檔案操作 | 25 | `cat`, `cp`, `mv`, `rm`, `ls`, `find` |
| 文字處理 | 20 | `grep`, `sed`, `awk`, `sort`, `tr`, `uniq` |
| 程式開發 | 12 | `make`, `yacc`, `lex`, `nm`, `strip` |
| 系統管理 | 15 | `ps`, `kill`, `id`, `chmod`, `df`, `du` |
| 通訊 | 8 | `mailx`, `write`, `mesg` |
| 計算 | 6 | `bc`, `expr`, `test` |
| Shell | 1 | `sh` (包含 93 個行為測試) |
| 其他 | 37 | `echo`, `printf`, `sleep`, `date`, `tee` |

## 交叉編譯細節

- **Host 編譯**: `cargo build --release` (使用主機 libc)
- **RISC-V 編譯**: `cargo build --release --no-default-features` (使用 `xv8-libc-compat`)
- **關鍵**: `libc` crate 在 Cargo.toml 中指向 `xv8rust/xv8-libc-compat`，在 RISC-V 上提供 syscall 包裝；在主機上則委派給真正的 `libc`

## 測試

```bash
cd posix && ./test.sh                    # 完整測試
cd posix && cargo build --release && \
  PATH="target/release:$PATH" sh tools/tests/test_sh_basic.sh   # shell 測試
cd posix && cargo build --release && \
  PATH="target/release:$PATH" sh tools/tests/test_tools_core.sh # 核心工具測試
```

## 相關文件

- [Wiki: POSIX](../_wiki/posix/POSIX.md)
- [Wiki: Shell](../_wiki/Shell.md)
- [xv8 AGENTS.md](../xv8/AGENTS.md)
- [工具內嵌文件](tools/src/bin/) (各工具對應的 `.md` 文件)
