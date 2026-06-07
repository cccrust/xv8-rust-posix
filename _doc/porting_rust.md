# Rust 標準函式庫移植狀態

> 參考：[Porting Rust standard library (OSDev Wiki)](https://wiki.osdev.org/Porting_Rust_standard_library)
> 目標架構：`riscv64gc-unknown-none-elf`
> 實作方式：`xv8-user-std` crate 作為 std overlay（非 rustc fork）

---

## 1. Guide — 自訂 Rust 工具鏈

### 1.1 Get sources / 1.2 Configuration / 1.3 Adding the target / 1.5 Add toolchain

| 項目 | 狀態 | 說明 |
|------|------|------|
| Fork rustc 原始碼 | ❌ 未完成 | 未 fork `rust-lang/rust`，使用上游已存在的 `riscv64gc-unknown-none-elf` target |
| 新增 `target_os = "xv8"` | ❌ 未完成 | 未在 `rustc_target/src/spec/` 中新增 xv8 target spec |
| 自訂 toolchain link | ❌ 未完成 | 不使用 `rustup toolchain link`，改以 `-Zbuild-std=core,alloc` 建構 std |
| `_start` symbol | ❌ 未完成 | 不由 std 提供 `_start`，由 xv8 kernel 的 `crt0` 負責 |
| CRT 物件檔 (`pre_link_objects`/`post_link_objects`) | ❌ 未完成 | 無自訂 CRT，使用 xv8 內建載入流程 |

**說明**：xv8 不走「fork rustc + 新增 OS target」的完整路線，而是利用 `riscv64gc-unknown-none-elf` 這個上游 bare-metal target，再以 xv8-user-std crate 疊加 OS 功能。這大幅降低了維護成本，但無法使用 `cfg(target_os = "xv8")` 等編譯期條件。

### 1.4 Adapt library/std (PAL)

| 項目 | 狀態 | 說明 |
|------|------|------|
| `library/std/src/sys/pal/xv8/` 目錄 | ❌ 未完成 | rustc 原始碼中無此目錄 |
| `alloc.rs` | ✅ xv8-user-std 等效 | `runtime.rs` — `ChunkAlloc` (64KB bump, `GlobalAlloc` trait) |
| `stdio.rs` | ✅ xv8-user-std 等效 | stdout/stderr 透過 `xv8_libc::write()` syscall |
| `fs.rs` | ✅ xv8-user-std 等效 | `fs::File`, `OpenOptions`, `canonicalize`, `Metadata` |
| `net.rs` | ✅ xv8-user-std 等效 | `TcpStream`, `TcpListener`, `lookup_host()` (DNS) |
| `thread.rs` | ✅ xv8-user-std 等效 | `thread::spawn`, `Builder`, `park`/`unpark` (futex) |
| `time.rs` | ✅ xv8-user-std 等效 | `Instant` (uptime), `Duration`, `sleep` |
| `process.rs` | ✅ xv8-user-std 等效 | `Command`, `Child`, `ExitStatus`, `Stdio` (piped) |
| `env.rs` | ✅ xv8-user-std 等效 | `args`, `vars`, `set_var` (unsafe), `current_dir` |
| `path.rs` | ✅ xv8-user-std 等效 | `PathBuf`, `Path` (依賴 `std::path`) |
| `sync.rs` | ✅ xv8-user-std 等效 | `Mutex` (futex 3-state), `Condvar` (futex counter), `Barrier`, `RwLock`, `OnceLock`, `LazyLock` |
| `mpsc.rs` | ✅ xv8-user-std 等效 | 管道實作 （`pipe()` syscall） |
| `ffi.rs` | ✅ xv8-user-std 等效 | `CString`, `OsString`（依賴 `std::ffi`） |
| `panic.rs` | ✅ xv8-user-std 等效 | `set_hook`/`take_hook`, panic → stderr + exit(1) |
| `os/unix/process.rs` | ✅ xv8-user-std 等效 | `CommandExt` (uid/gid no-op trait) |
| `os/unix/net.rs` | ✅ xv8-user-std 等效 | `UnixStream` (pipe-based `pair()`, `dup()`), `UnixListener`/`UnixDatagram` → `Unsupported` |

---

## 2. Making the standard library functional

### 2.1 Memory allocator

| 項目 | 狀態 | 說明 |
|------|------|------|
| `GlobalAlloc` trait 實作 | ✅ 已完成 | `ChunkAlloc`：64KB chunk bump allocator |
| 縮減 sbrk syscall 次數 | ✅ 已完成 | 每次分配從 chunk 中 bump，chunk 用完才呼叫 sbrk |
| `alloc`/`dealloc` | ✅ 已完成 | bump allocator（不回收），`dealloc` 為 no-op |

**實作位置**：`xv8rust/xv8-user-std/src/runtime.rs`

### 2.2 Thread local storage

| 項目 | 狀態 | 說明 |
|------|------|------|
| Global statics（單執行緒） | — 不使用 | xv8 支援多執行緒，不可用 |
| OS APIs（`thread_local_key.rs`） | — 不使用 | 未實作此較慢方案 |
| Native ELF TLS | ✅ 已完成 | `#[thread_local]` + kernel `clone_tls` + tp 暫存器 |
| `has_thread_local: true` | ✅ 上游已支援 | `riscv64gc-unknown-none-elf` target 原生開啟 |

**實作細節**：
- Kernel 在 `clone` syscall 中配置 TLS 區域並設定 `tp` 暫存器
- 使用者態程式透過 `#[thread_local]` 即可使用 thread-local 變數
- 不使用 `static_local` fallback（與 wasm/uefi 不同）

### 2.3 Basic printing

| 項目 | 狀態 | 說明 |
|------|------|------|
| `Stdin` | ✅ 已完成 | 從 fd 0 讀取（`xv8_libc::read()`） |
| `Stdout` | ✅ 已完成 | 寫入 fd 1（`xv8_libc::write()`） |
| `Stderr` | ✅ 已完成 | 寫入 fd 2（`xv8_libc::write()`） |
| `panic_output()` | ✅ 已完成 | 回傳 `Stderr::new()`，panic 訊息輸出到 stderr |
| `is_ebadf()` | ✅ 已完成 | 檢查 `EBADF` 錯誤碼 |

**實作位置**：`xv8rust/xv8-user-std/src/io.rs`

---

## 3. Runtime — Integrating a crate

| 項目 | 狀態 | 說明 |
|------|------|------|
| runtime crate 加入 std 依賴 | ❌ 未完成 | 未將 `xv8-user-std` 以 `rustc-dep-of-std` 方式加入 rustc std |
| `rustc-dep-of-std` feature | ❌ 未完成 | 無需此功能，xv8-user-std 為獨立 crate |
| 全域變數重複問題 | — 不適用 | xv8-user-std 非 std 內部依賴，無 linkage 衝突 |

**說明**：OSDev 指南建議將 runtime crate 以 `[target.'cfg(target_os = "myos")'.dependencies]` 加入 std 的 `Cargo.toml`。xv8 的做法不同：使用者直接依賴 `xv8-user-std` crate，它會 re-export `std::*` 並覆蓋特定模組。這不需要修改 rustc 原始碼。

---

## 4. 綜合評估

### 已完成的核心功能

| 功能 | 對應 std 模組 | xv8-user-std 實作 |
|------|--------------|------------------|
| 記憶體分配 | `alloc::GlobalAlloc` | `ChunkAlloc`（bump allocator） |
| 執行緒 | `std::thread` | spawn, join, park/unpark, Builder, name |
| 同步 | `std::sync` | Mutex, Condvar, Barrier, RwLock, OnceLock, LazyLock, mpsc |
| 檔案 I/O | `std::fs`, `std::io` | File, OpenOptions, canonicalize, copy, empty/repeat/sink |
| 網路 | `std::net` | TcpStream, TcpListener, lookup_host |
| 程序 | `std::process` | Command, Child, Stdio（piped）, ExitStatus |
| 環境變數 | `std::env` | args, vars, set_var (unsafe), current_dir, temp_dir |
| 時間 | `std::time` | Instant, Duration, sleep |
| 恐慌處理 | `std::panic` | set_hook/take_hook, panic → stderr |
| Unix 擴充 | `std::os::unix` | CommandExt, UnixStream (pipe), UnixListener, UnixDatagram |
| 非同步 I/O | tokio compat | AsyncTcpStream, AsyncTcpListener, epoll reactor |

### 尚未完成的功能

| 功能 | 困難度 | 說明 |
|------|--------|------|
| Fork rustc 新增 `target_os = "xv8"` | 🔴 高 | 需維護 rustc fork，追蹤 upstream 變更 |
| 完整 PAL 實作 | 🔴 高 | 需在 rustc 原始碼中建立 `sys/pal/xv8/` 目錄 |
| 自訂 toolchain | 🟡 中 | `rustup toolchain link` + 每次 rustc 更新需重建 |
| `rustc-dep-of-std` runtime crate | 🟡 中 | 需處理 crate 重複與 linkage 問題 |
| 動態連結 | 🔴 高 | 目前 xv8 僅支援靜態連結 |
| 共享記憶體 | 🟡 中 | 無 `mmap(MAP_SHARED)` 或 `shm` syscall |
| 信號處理 | 🟡 中 | 無完整 signal 實作 |
| 使用者/群組 | 🟢 低 | `uid`/`gid` 為 no-op，無實際 ACL |

### 測試覆蓋

| 類別 | 數量 | 狀態 |
|------|------|------|
| Core（process, thread, sync, fs） | 9 | ✅ 全數通過 |
| Network（TCP, DNS, echo） | 6 | ✅ 全數通過 |
| Async（epoll, tokio compat） | 3 | ✅ 全數通過 |
| 總計 | 18 | ✅ 全數通過（QEMU） |

---

## 5. 結論與建議

### 現行方案（輕量 overlay）

**優點**：
- 無需 fork rustc，使用上游工具鏈
- `cargo build -Zbuild-std=core,alloc` 即可建構
- 快速迭代，18 個 QEMU 測試全數通過
- 與 POSIX 工具鏈共用同一套 syscall ABI

**缺點**：
- 無法使用 `cfg(target_os = "xv8")`
- 非標準的 std 整合方式
- 某些 crate 可能無法正確識別平台

### 完整移植路線（若需 strict OSDev 指南遵循）

1. Fork `rust-lang/rust`
2. 在 `rustc_target/` 中新增 `riscv64gc-unknown-xv8` target spec
3. 在 `library/std/src/sys/pal/` 中建立 `xv8/` 目錄
4. 將 `xv8-libc` 改為 `rustc-dep-of-std` 依賴
5. 產出自訂 toolchain：`rustup toolchain link dev-xv8 ...`
6. 全面測試

---

> 最後更新：2026-06-07
> 參考版本：xv8 v3.8

