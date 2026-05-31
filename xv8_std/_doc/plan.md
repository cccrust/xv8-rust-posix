# xv8-std Plan：自訂 std overlay 讓 posix/tools 編譯到 xv8

## 目標

讓 `posix/tools/` **不修改任何一行** 就能在 xv8（riscv64gc-unknown-none-elf）上編譯執行。

## 核心架構

```
xv8_std/
├── user.ld                # linker script（從 xv8/user.ld 複製）
├── xv8-libc/              # no_std 獨立 ecall wrapper
│   └── src/
│       ├── raw.rs         # Syscall enum + raw ecall (asm)
│       ├── args.rs        # Args：從暫存器讀 argv
│       └── lib.rs         # Fd, SysError, check(), OpenFlag, re-export
│
└── xv8-user-std/          # std trait overlay（依賴 xv8-libc, 不依賴 user crate）
    ├── runtime.rs         # _start, panic_handler, global_allocator, lang_start
    ├── io.rs              # Read, Write, BufRead, Seek + Stdin/Stdout/Stderr/BufReader
    ├── path.rs            # Path, PathBuf
    ├── fs.rs              # File, OpenOptions, Metadata, read_to_string, read, read_link
    ├── env.rs             # args(), vars(), current_dir(), set_current_dir()
    ├── process.rs         # exit(), Command (with spawn/output), Child, ChildStdout, Output
    ├── time.rs            # Duration, SystemTime, Instant
    ├── thread.rs          # sleep (stub)
    ├── ffi.rs             # CStr, CString, OsStr
    └── os/unix/fs.rs      # MetadataExt, PermissionsExt, symlink
```

**關鍵：** `xv8-user-std` 依賴 `xv8-libc`，**不依賴** `user` crate。

## posix/tools 整合方式

在 `posix/tools/Cargo.toml` 中以 target 條件取代 std：

```toml
[target.riscv64gc-unknown-none-elf.dependencies]
std = { package = "xv8-user-std", path = "../../xv8_std/xv8-user-std" }
```

加上 linker script 和 build-std：

```toml
# posix/.cargo/config.toml
[target.riscv64gc-unknown-none-elf]
rustflags = ["-C", "link-arg=-T../xv8_std/user.ld"]

[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
```

## 實作範圍

### v0.2（已完成）
- 架構重整，解除 user crate 依賴
- xv8-libc：補 chdir, Args, OpenFlag
- xv8-user-std：全部模組改為依賴 xv8-libc

### v0.3（已完成）
- posix/tools 整合：135 工具，125 編譯成功
- 執行期支援：_start, panic_handler, global_allocator (sbrk), lang_start
- 完整 Io：Read/Write/BufRead blanket impls for Box&lt;T&gt; 和 &mut T, IsTerminal
- 路徑：OsStr 類型, PathBuf::Deref, file_stem
- 檔案系統：FileType, read, read_link, OpenOptions::create_new, seek
- 行程：process::id, Command stdio 設定, ChildStdout, Output
- 時間：Instant, Duration 補完
- 額外：HashMap/HashSet (via hashbrown), char/iter/error modules
- 工具修正：date.rs (i64→i32), who.rs (補 cfg(not(unix)) fallback)

### v0.4（規劃中）
- 補 libc 替代 syscall（signals, utsname），攻克 10 個 libc 依賴工具
- 實作 fs::read_link, process::spawn 的真實 syscall
- QEMU 測試 Phase 1 工具
- clock_gettime syscall

## 當前狀態

| 類別 | 數量 |
|---|---|
| 工具總數 | 135 |
| 編譯成功 | 125 |
| libc 依賴失敗 | 10 (bg, fg, ipcrm, ipcs, jobs, mkfifo, newgrp, su, uname, vi) |

## 限制

- 第三方 crates（如 `regex`）仍然依賴編譯器內建 `std`，無法使用
- 只有直接使用 `std::*` 的程式碼能被 overlay 取代
- 10 個工具使用 `libc` crate 直接呼叫系統 API，需補 wrapper
- 許多 stub 回傳 `Unsupported`，執行時期會失敗
