# posix — POSIX 使用者工具集

## 目標

實作一套 POSIX.1-2008 相容的使用者空間工具集，
**可在 Linux、macOS、Windows（MSYS2/Cygwin）、xv8-rust-posix（RISC-V）上編譯並執行**。

這些工具是真正的 POSIX 工具，不是僅為 xv8 設計的附屬品。
任何符合 POSIX 標準的系統都應該能編譯和使用它們。

## 第一優先：主機原生執行

**Linux、macOS、Windows（MSYS2/Cygwin）為第一級目標。**
xv8 為次要（嵌入式）目標。

| 目標 | 平臺 | 說明 |
|------|------|------|
| `x86_64-apple-darwin` | macOS Intel | 開發用，可直接執行與測試 |
| `aarch64-apple-darwin` | macOS Apple Silicon | 開發用 |
| `x86_64-unknown-linux-gnu` | Linux | 通用 Linux |
| MSYS2 / MinGW-w64 (`*-pc-windows-gnu`) | Windows | POSIX 環境模擬 |
| Cygwin (`*-pc-cygwin`) | Windows | POSIX 環境模擬 |
| `riscv64gc-unknown-none-elf` | xv8 RISC-V | 嵌入目標（`no_std`） |

開發流程：
1. 在 macOS/Linux/Windows 上撰寫、編譯、測試工具（使用 `std`）
2. 確認行為與 GNU/BSD coreutils 一致
3. 針對 xv8 目標做交叉編譯與驗證

## 平臺抽象層

共用邏輯與 I/O 層分離：

```
libposix/src/
├── lib.rs
├── io.rs       # 統一 io::Read / io::Write trait
├── fmt.rs      # 格式化工具
├── opt.rs      # POSIX getopt 實作
├── platform/
│   ├── mod.rs
│   ├── unix.rs  # cfg(unix) — 透過 std 呼叫 POSIX API（Linux/macOS/Cygwin）
│   ├── msvc.rs  # cfg(all(windows, not(target_vendor = "pc"))) — MSYS2/MinGW-w64
│   └── xv8.rs   # cfg(target_os = "none") — 透過 xv8 syscall
```

> Windows/MSYS2 與 MinGW 使用 `cfg(windows)` 條件編譯，透過 Rust `std` 進行 POSIX-like 操作。
> Rust `std::path::Path` 自動處理路徑分隔符號，工具程式碼不需直接操作路徑字串。
> Cygwin 的 Rust 目標為 `x86_64-pc-cygwin` 屬 `cfg(unix)`，與 Unix 同一分支。

工具程式碼本身**不包含 `#[cfg]`**，僅依賴 `libposix::*` 的統一介面。
平臺差異完全封裝在 `libposix::platform` 中。

## 專案結構

```
os/posix/
├── Cargo.toml          # workspace root
├── _doc/
│   └── plan.md         # 本文件
├── libposix/           # 共用函式庫
│   ├── Cargo.toml
│   └── src/ ...
└── src/
    ├── cat.rs
    ├── ls.rs
    ├── cp.rs
    ├── mv.rs
    ├── rm.rs
    ├── mkdir.rs
    ├── rmdir.rs
    ├── touch.rs
    ├── ln.rs
    ├── chmod.rs
    ├── chown.rs
    ├── echo.rs
    ├── printf.rs
    ├── true.rs
    ├── false.rs
    ├── test.rs
    ├── wc.rs
    ├── sort.rs
    ├── uniq.rs
    ├── head.rs
    ├── tail.rs
    ├── od.rs
    ├── diff.rs
    ├── cmp.rs
    ├── grep.rs
    ├── sed.rs
    ├── cut.rs
    ├── tr.rs
    ├── tee.rs
    ├── xargs.rs
    ├── env.rs
    ├── printenv.rs
    ├── basename.rs
    ├── dirname.rs
    ├── whoami.rs
    ├── id.rs
    ├── uname.rs
    ├── hostname.rs
    ├── date.rs
    ├── sleep.rs
    ├── kill.rs
    ├── nice.rs
    ├── nohup.rs
    ├── du.rs
    ├── df.rs
    ├── ps.rs
    ├── stty.rs
    └── sh.rs
```

## 實作階段

### Phase 1：基礎架構 + 最小工具集

建立 Cargo workspace + libposix 平臺抽象層，實作第一批工具：

| 工具 | POSIX 強制選項 | 依賴 |
|------|---------------|------|
| `echo` | 無 | 基本 |
| `true` | 無 | 基本 |
| `false` | 無 | 基本 |
| `yes` | 無 | 基本 |
| `cat` | `-u` | 檔案 I/O |
| `wc` | `-c`, `-l`, `-w` | 檔案 I/O |
| `basename` | 無 | 字串處理 |
| `dirname` | 無 | 字串處理 |
| `sleep` | `-s` | clock |
| `kill` | `-l`, `-s` | signal |
| `uname` | `-a`, `-m`, `-n`, `-r`, `-s`, `-v` | uname |
| `printenv` | 無 | 環境變數 |
| `env` | `-i`, `-u` | 環境變數 |
| `whoami` | 無 | 使用者 |
| `id` | `-G`, `-g`, `-n`, `-r`, `-u` | 使用者 |
| `hostname` | 無 | 主機名稱 |

交付：`cargo build` 可在 Linux/macOS 上編譯，每支工具支援 POSIX 強制選項。

### Phase 2：檔案操作工具

| 工具 | POSIX 強制選項 |
|------|---------------|
| `ls` | `-a`, `-C`, `-d`, `-F`, `-i`, `-l`, `-m`, `-p`, `-q`, `-r`, `-R`, `-s`, `-x`, `-1` |
| `cp` | `-f`, `-i`, `-p`, `-R` |
| `mv` | `-f`, `-i` |
| `rm` | `-f`, `-i`, `-R` |
| `mkdir` | `-p`, `-m` |
| `rmdir` | `-p` |
| `ln` | `-f`, `-s` |
| `touch` | `-a`, `-c`, `-m`, `-r`, `-t` |
| `chmod` | `-R` |
| `chown` | `-R`（POSIX optional） |

### Phase 3：文字處理工具

`head`, `tail`, `sort`, `uniq`, `cut`, `tr`, `tee`, `od`, `cmp`, `diff`

### Phase 4：搜尋與過濾工具

`grep`, `sed`, `xargs`

### Phase 5：系統工具

`ps`, `du`, `df`, `stty`, `test`, `nice`, `nohup`, `date`

### Phase 6：Shell

`sh` — POSIX shell（支援內建指令、控制流程、工作控制、I/O 重導）

### Phase 7：進階工具

`find`, `tar`, `patch`, `comm`, `fold`, `fmt`, `nl`, `expand`, `unexpand`

## 測試策略

| 層級 | 方法 |
|------|------|
| 單元測試 | `cargo test`（libposix 各模組） |
| 行為驗證 | 每支工具比對 GNU/BSD coreutils 輸出 |
| 回歸測試 | 固定輸入 → 比對預期輸出（快照測試） |
| xv8 整合 | QEMU 上跑 `_testrunner` 驗證 |

所有測試**優先在主機（macOS/Linux）上執行**，xv8 僅作為次要驗證。

## 設計原則

1. **POSIX 第一** — 以 POSIX.1-2008 Shell & Utilities 為規範，非 GNU 擴充
2. **原生可執行** — Linux/macOS/Windows 上可直接 `cargo build && cargo run --bin cat`
3. **路徑抽象** — 使用 `std::path::Path` 處理路徑，不用字串拼接；Windows MSYS2 下自動適應正斜線
4. **平臺抽象** — 工具邏輯零平臺依賴，所有差異封裝在 `libposix::platform`
5. **零外部相依** — 純 Rust stdlib，維持可移植性
6. **逐步擴充** — 從 POSIX 最小選項開始，逐步補齊
