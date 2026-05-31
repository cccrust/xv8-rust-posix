# xv8-std Plan：自訂 std overlay 讓 posix/tools 編譯到 xv8

## 目標

讓 `posix/tools/` **不修改任何一行** 就能在 xv8（riscv64gc-unknown-none-elf）上編譯執行。

## 核心架構

```
xv8_std/
├── xv8-libc/         # no_std 獨立 ecall wrapper
│   └── src/
│       ├── raw.rs    # Syscall enum + raw ecall (asm)
│       ├── args.rs   # Args：從暫存器讀 argv
│       └── lib.rs    # Fd, SysError, check(), OpenFlag, re-export
│
└── xv8-user-std/     # std trait overlay（依賴 xv8-libc, 不依賴 user crate）
    ├── io.rs         # Read, Write, BufRead, Seek traits + 實作
    ├── path.rs       # Path, PathBuf
    ├── fs.rs         # File, OpenOptions, Metadata, read_to_string
    ├── env.rs        # args(), current_dir(), set_current_dir()
    ├── process.rs    # exit(), Command (stub)
    ├── time.rs       # Duration, SystemTime
    ├── thread.rs     # sleep (stub)
    ├── ffi.rs        # CStr, CString
    └── os/unix/fs.rs # MetadataExt, PermissionsExt
```

**關鍵：** `xv8-user-std` 依賴 `xv8-libc`，**不依賴** `user` crate。`user` crate 使用 `std` + edition 2024，無法編譯到 `riscv64gc-unknown-none-elf`。

## posix/tools 整合方式

在 `posix/tools/Cargo.toml` 中以 target 條件取代 std：

```toml
[target.riscv64gc-unknown-none-elf.dependencies]
std = { package = "xv8-user-std" }
```

這讓 `use std::io::Read` 解析為 `xv8_user_std::io::Read`。

## 實作範圍

### v0.2（已完成）
- 架構重整，解除 user crate 依賴
- xv8-libc：補 chdir, Args, OpenFlag
- xv8-user-std：全部模組改為依賴 xv8-libc
- 清除空目錄 xv8-std-overlay

### v0.3（下一步）
- posix/tools 整合
- 文字處理工具（cat, head, tail, grep, sort, uniq, tr, tee, wc, cut, od, cmp, expand, unexpand, fmt, nl, fold, paste, strings）編譯驗證

### 後續版本
- `std::fs`（read_dir, MetadataExt）— ls 等工具
- `std::process::Command`（fork/exec）— sh, xargs, nohup
- `std::time::SystemTime` — touch
- `std::thread::sleep` — sleep

## 限制

- 第三方 crates（如 `regex`）仍然依賴編譯器內建 `std`，無法使用
- 只有直接使用 `std::*` 的程式碼能被 overlay 取代
