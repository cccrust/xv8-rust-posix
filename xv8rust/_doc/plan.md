# xv8-std Plan：自訂 std overlay 讓 posix/tools 編譯到 xv8

## 目標

讓 `posix/tools/` **不修改任何一行** 就能在 xv8（riscv64gc-unknown-none-elf）上編譯執行。

## 核心架構

```
xv8rust/
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
std = { package = "xv8-user-std", path = "../../xv8rust/xv8-user-std" }
```

加上 linker script 和 build-std：

```toml
# posix/.cargo/config.toml
[target.riscv64gc-unknown-none-elf]
rustflags = ["-C", "link-arg=-T../xv8rust/user.ld"]

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

### v0.4（已完成）
- 補 `xv8-libc-compat`：在 riscv64 提供最小 `libc` 相容層，攻克 10 個 libc 依賴工具
- 對齊 `utsname`、`termios`、`winsize`、`passwd`、`group`、`shmid_ds`、`semid_ds` 等 ABI
- 補 `kill`、`waitpid`、`uname`、`mkfifo`、`setuid`、`setregid`、`tcgetattr`、`tcsetattr`、`ioctl`、`sysconf`
- `posix/tools` 目標平台編譯驗證通過：135/135
- `ipcrm` 補齊 `null_mut::<c_void>()` 型別註記，避免泛型指標推導失敗

### v0.5（已完成）
- 整理 `xv8rust` 目前仍可消除的警告
- 實作 `fs::read_link` 的真實 syscall 路徑
- 實作 `process::Command::spawn` 的真實 syscall 路徑
- 持續補齊 runtime 行為，讓更多工具不只「能編譯」，也能「能執行」

### v0.6（已完成）
- 建立根目錄 `test.sh`，串起 `posix`、`xv8rust`、`xv8` 三段驗證
- 加入 `rustc-wrapper`，讓 `posix/tools` 的 xv8 target bin 自動帶上 `no_main`
- 修正 `basic.rs` 的工具路徑解析，避免測到系統 `mailx`
- 修正 `ipcrm.rs` 的 shm/sem 指標型別，讓 target build 穩定通過
- 完成整體驗證：`posix` host tests、`posix` target build、`xv8rust` target build、`xv8` integration tests 全過

### v1.0（已完成）
- 供應商化 crossterm 0.29.0 到 `xv8rust/crossterm/`
- 新增 riscv64 terminal backend（ioctl CONSOLE_SET_RAW, 80×24 預設視窗）
- `spin::Once` 取代 `parking_lot::Once` for riscv64
- xv8-libc 新增 `ioctl`、`tcgetattr`、`tcsetattr` syscall wrappers
- 修正 sh.rs heredoc（多行累積、quoted delimiter tokenizer 缺字、pipeline heredoc 內容遺失）
- crossterm 可選化：feature gate + `required-features` on vi/vim
- `test.sh` riscv64 build 使用 `--no-default-features`
- 全測試通過：cargo tests 214/214、shell 33/33、core tools 21/21

## 當前狀態

| 類別 | 數量 |
|---|---|
| 工具總數 | 135 |
| 編譯成功 | 135 |
| libc 依賴失敗 | 0 |
| 全鏈路驗證 | 通過 |
| vi/vim host 可用 | 是 |
| riscv64 crossterm | 未驗證（xv8-user-std 不支援完整 std） |

## 限制

- 第三方 crates（如 `regex`）仍然依賴編譯器內建 `std`，無法使用
- 只有直接使用 `std::*` 的程式碼能被 overlay 取代
- `libc` 相關 API 在 riscv64 由 `xv8-libc-compat` 提供最小相容層；部分函式仍是 stub，僅保證編譯
- 許多 stub 回傳 `Unsupported`，執行時期會失敗

## 下一步：async / web runtime

- 已新增 `xv8rust/xv8-async`，提供單執行緒 async runtime / reactor scaffold
- 已新增 `xv8rust/xv8-axum-smoke`，先把本地 `tokio` / `axum` 的 host smoke path 跑通
- 下一步是把同一條 smoke path 移到 xv8 target，並補齊 `tokio::net` / `tokio::time` / `tokio::spawn` 需要的剩餘面
- 若遇到 upstream crate 的額外要求，優先把缺口回寫到 `xv8-user-std`，再評估是否需要新的 shim crate
